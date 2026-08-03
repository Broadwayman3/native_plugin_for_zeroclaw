#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Bot UI Handlers
Domain logic handlers for Telegram callback queries and text messages.
Delegates to bot_ui_utils for payload builders and i18n for translations.
Returns lists of (method, payload) tuples for the listener to dispatch.
"""

import os
import sys
import random
from typing import Dict, Any, List, Tuple

SCRIPT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from pos_core.bot_ui_utils import (
    generate_lang_inline_keyboard,
    is_btn_click,
    parse_pos_order_input,
    build_send_message_payload,
    build_answer_callback_payload,
    MERCHANT_WALLET_PUBKEY,
)
import pos_core.bot_ui_utils as _bot_ui
from pos_core.i18n import (
    get_localized_confirmation,
    get_main_reply_keyboard,
    get_cancel_invoice_inline_keyboard,
    get_refund_checkpoint_inline_keyboard,
    t,
    format_itemized_receipt,
)
from pos_core.db import (
    DB_PATH,
    get_db_connection,
    create_invoice_record,
    cancel_invoice_record,
    get_sales_summary_stats,
    create_squads_proposal,
    update_squads_proposal_status,
)
from pos_core.solana_pay import generate_secure_reference_key, generate_solana_pay_url, initiate_refund_request
from pos_core.price_feed import get_multitier_fiat_rate
from pos_core.formatters import generate_solana_pay_qr_image_url, generate_telegram_photo_payload
from sanitizer import sanitize_external_input


def handle_callback_query(cb: Dict[str, Any], session: Dict[str, Any], db_path: str = DB_PATH) -> List[Tuple[str, Dict[str, Any]]]:
    cb_id = cb.get("id", "")
    data_str = cb.get("data", "")
    msg = cb.get("message", {})
    chat_id = (msg.get("chat") or msg).get("id", 0) if isinstance(msg, dict) else 0

    payloads: List[Tuple[str, Dict[str, Any]]] = []

    if data_str.startswith("set_lang_"):
        new_lang = data_str.replace("set_lang_", "")
        session["lang"] = new_lang
        session["user_set"] = True
        session["state"] = "idle"
        session["draft_items"] = None
        conf_msg = get_localized_confirmation(new_lang)
        payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "Language Changed!")))
        payloads.append(("sendMessage", build_send_message_payload(chat_id, conf_msg, reply_markup=get_main_reply_keyboard(new_lang))))

    elif data_str.startswith("cancel_invoice_"):
        inv_id = data_str.replace("cancel_invoice_", "")
        rowcount = cancel_invoice_record(inv_id, db_path=db_path)
        if rowcount > 0:
            payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "Voided!")))
            void_msg = t("void_confirmed", session["lang"], escape_markdown=False, invoice_id=inv_id)
            payloads.append(("sendMessage", build_send_message_payload(chat_id, void_msg, reply_markup=get_main_reply_keyboard(session["lang"]))))
        else:
            payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "409 Conflict: Already voided or paid", show_alert=True)))
            payloads.append(
                (
                    "sendMessage",
                    build_send_message_payload(
                        chat_id,
                        t("invoice_already_cancelled", session["lang"], escape_markdown=False, invoice_id=inv_id),
                        reply_markup=get_main_reply_keyboard(session["lang"]),
                    ),
                )
            )

    elif data_str.startswith("approve_refund_"):
        from_id = int(cb.get("from", {}).get("id", 0))
        if from_id != _bot_ui.MANAGER_TELEGRAM_ID:
            payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "⛔ Unauthorized Manager ID", show_alert=True)))
            payloads.append(("sendMessage", build_send_message_payload(chat_id, t("unauthorized_approve", session["lang"], escape_markdown=False))))
            return payloads

        try:
            proposal_index = int(data_str.replace("approve_refund_", "").strip())
        except ValueError:
            payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "⚠️ Invalid callback data", show_alert=True)))
            return payloads

        conn = get_db_connection(db_path)
        try:
            update_squads_proposal_status(conn, proposal_index, "approved")
            cursor = conn.cursor()
            cursor.execute(
                "UPDATE invoices SET status = 'refund_proposed_squads_v4', updated_at = CURRENT_TIMESTAMP"
                " WHERE id = (SELECT invoice_id FROM squads_proposals WHERE proposal_index = ?)",
                (proposal_index,),
            )
            conn.commit()
        finally:
            conn.close()
        payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "Approved!")))
        payloads.append(
            (
                "sendMessage",
                build_send_message_payload(
                    chat_id,
                    t("squads_refund_approved", session["lang"], escape_markdown=False, proposal_index=proposal_index),
                    reply_markup=get_main_reply_keyboard(session["lang"]),
                ),
            )
        )

    elif data_str.startswith("reject_refund_"):
        from_id = int(cb.get("from", {}).get("id", 0))
        if from_id != _bot_ui.MANAGER_TELEGRAM_ID:
            payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "⛔ Unauthorized Manager ID", show_alert=True)))
            payloads.append(("sendMessage", build_send_message_payload(chat_id, t("unauthorized_reject", session["lang"], escape_markdown=False))))
            return payloads

        try:
            proposal_index = int(data_str.replace("reject_refund_", "").strip())
        except ValueError:
            payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "⚠️ Invalid callback data", show_alert=True)))
            return payloads

        conn = get_db_connection(db_path)
        try:
            update_squads_proposal_status(conn, proposal_index, "rejected")
            cursor = conn.cursor()
            cursor.execute(
                "UPDATE invoices SET status = 'paid', updated_at = CURRENT_TIMESTAMP"
                " WHERE id = (SELECT invoice_id FROM squads_proposals WHERE proposal_index = ?)",
                (proposal_index,),
            )
            conn.commit()
        finally:
            conn.close()
        payloads.append(("answerCallbackQuery", build_answer_callback_payload(cb_id, "Rejected")))
        payloads.append(
            (
                "sendMessage",
                build_send_message_payload(
                    chat_id,
                    t("squads_refund_rejected", session["lang"], escape_markdown=False, proposal_index=proposal_index),
                    reply_markup=get_main_reply_keyboard(session["lang"]),
                ),
            )
        )

    return payloads


def handle_text_message(msg: Dict[str, Any], session: Dict[str, Any], db_path: str = DB_PATH) -> List[Tuple[str, Dict[str, Any]]]:
    chat_id = msg.get("chat", {}).get("id", 0)
    payloads: List[Tuple[str, Dict[str, Any]]] = []
    user_lang = session.get("lang", "uk")

    raw_text = (msg.get("text") or "").strip()
    if not raw_text:
        return payloads

    text = sanitize_external_input(raw_text)
    if not text:
        return payloads

    text_lower = text.lower()

    if text_lower in ["/start", "меню", "menu"]:
        session["state"] = "idle"
        session["draft_items"] = None
        welcome_msg = t("welcome", user_lang, escape_markdown=False)
        payloads.append(
            ("sendMessage", build_send_message_payload(chat_id, welcome_msg, parse_mode="Markdown", reply_markup=get_main_reply_keyboard(user_lang)))
        )
        return payloads

    if is_btn_click(text, "btn_lang") or any(
        kw in text_lower for kw in ["13 мов", "language", "idioma", "sprache", "langue", "lingua", "język", "dil", "言語", "语言", "भाषा", "لغة"]
    ):
        session["state"] = "idle"
        session["draft_items"] = None
        select_lang_msg = t("select_lang", user_lang, escape_markdown=False)
        payloads.append(
            ("sendMessage", build_send_message_payload(chat_id, select_lang_msg, parse_mode="Markdown", reply_markup=generate_lang_inline_keyboard()))
        )
        return payloads

    if is_btn_click(text, "btn_custom") or any(
        kw in text_lower for kw in ["custom", "довільн", "personalizado", "eingeben", "montant", "importo", "kwotę", "tutar", "入力", "自定义", "दर्ज", "مخصص"]
    ):
        session["state"] = "awaiting_custom_amount"
        session["draft_items"] = None
        custom_msg = t("custom_help", user_lang, escape_markdown=False)
        payloads.append(
            ("sendMessage", build_send_message_payload(chat_id, custom_msg, parse_mode="Markdown", reply_markup=get_main_reply_keyboard(user_lang)))
        )
        return payloads

    if is_btn_click(text, "btn_quick_uah") or any(
        kw in text_lower
        for kw in ["quick", "200 uah", "швидкий", "szybki", "schnell", "rápido", "rapide", "rapido", "hızlı", "クイック", "快速", "त्वरित", "سريع"]
    ):
        session["state"] = "idle"
        session["draft_items"] = None
        fiat_amt = 200.0
        fiat_curr = "UAH"
        rate_info = get_multitier_fiat_rate(fiat_curr)
        rate = rate_info.get("rate", 41.5)
        usdc_amt = round(fiat_amt / rate, 2)
        inv_id = f"INV-{random.randint(200, 999)}"
        ref_key = generate_secure_reference_key()
        create_invoice_record(
            {"id": inv_id, "reference_pubkey": ref_key, "fiat_currency": fiat_curr, "fiat_amount": fiat_amt, "usdc_amount": usdc_amt}, db_path=db_path
        )

        item_desc = t("default_item", lang=user_lang, escape_markdown=False) + f" {fiat_amt} {fiat_curr}"
        receipt_text = format_itemized_receipt(
            inv_id, item_desc, 0.0, usdc_amt, lang=user_lang, fiat_currency=fiat_curr, fiat_amount=fiat_amt, exchange_rate=rate
        )
        solana_url = generate_solana_pay_url(MERCHANT_WALLET_PUBKEY, usdc_amt, ref_key)
        qr_photo_url = generate_solana_pay_qr_image_url(solana_url, size=300)
        keyboard = get_cancel_invoice_inline_keyboard(inv_id, lang=user_lang)

        photo_payload = generate_telegram_photo_payload(str(chat_id), qr_photo_url, receipt_text, reply_markup=keyboard)
        payloads.append(("sendPhoto", photo_payload))
        return payloads

    if is_btn_click(text, "btn_sales") or any(
        kw in text_lower
        for kw in ["звіт", "sales", "vendas", "resumen", "übersicht", "résumé", "riepilogo", "podsumowanie", "özeti", "売上", "销售", "बिक्री", "ملخص"]
    ):
        session["state"] = "idle"
        session["draft_items"] = None
        stats = get_sales_summary_stats(db_path=db_path)
        summary_msg = (
            f"📊 *ZeroClaw POS Sales Summary ({user_lang.upper()})*\n"
            "───────────────────────────\n"
            f"• Paid Invoices : *{stats.get('total_paid_invoices', 0)}*\n"
            f"• Total Revenue : *${stats.get('total_sales_usdc', 0.0):.2f} USDC*\n"
            f"• Pending       : *{stats.get('total_pending_invoices', 0)}*\n"
            "───────────────────────────\n"
            "• WAL Mode State : `Active`\n"
            "• Server Status  : `OPERATIONAL`"
        )
        payloads.append(
            ("sendMessage", build_send_message_payload(chat_id, summary_msg, parse_mode="Markdown", reply_markup=get_main_reply_keyboard(user_lang)))
        )
        return payloads

    if is_btn_click(text, "btn_refund") or any(
        kw in text_lower
        for kw in ["рефанд", "refund", "reembolso", "rückerstattung", "remboursement", "rimborso", "zwrot", "iade", "返金", "退款", "रिफंड", "استرداد"]
    ):
        session["state"] = "awaiting_refund_invoice"
        session["draft_items"] = None
        refund_prompt = t("refund_prompt", user_lang, escape_markdown=False)
        payloads.append(
            ("sendMessage", build_send_message_payload(chat_id, refund_prompt, parse_mode="Markdown", reply_markup=get_main_reply_keyboard(user_lang)))
        )
        return payloads

    if session.get("state") == "awaiting_refund_invoice":
        inv_id = text.replace("#", "").strip().upper()
        session["state"] = "idle"
        session["draft_items"] = None
        conn = get_db_connection(db_path)
        try:
            success = initiate_refund_request(conn, inv_id)
            if not success:
                error_text = t("refund_error", user_lang, escape_markdown=False, error_msg=f"{inv_id} not found or already refunded")
                payloads.append(("sendMessage", build_send_message_payload(chat_id, error_text, reply_markup=get_main_reply_keyboard(user_lang))))
                return payloads

            cursor = conn.cursor()
            cursor.execute("SELECT customer_address, usdc_amount FROM invoices WHERE id = ?", (inv_id,))
            row = cursor.fetchone()
            if row:
                recipient_pubkey = row[0]
                amount_usdc = row[1]
            else:
                recipient_pubkey = MERCHANT_WALLET_PUBKEY
                amount_usdc = 0.0

            try:
                proposal_index = create_squads_proposal(conn, inv_id, recipient_pubkey or MERCHANT_WALLET_PUBKEY, amount_usdc or 0.0)
            except Exception:
                conn.rollback()
                cursor.execute("UPDATE invoices SET status = 'paid', updated_at = CURRENT_TIMESTAMP WHERE id = ?", (inv_id,))
                conn.commit()
                error_text = t("refund_error", user_lang, escape_markdown=False, error_msg=f"Failed to create Squads v4 proposal for {inv_id}")
                payloads.append(("sendMessage", build_send_message_payload(chat_id, error_text, reply_markup=get_main_reply_keyboard(user_lang))))
                return payloads

            keyboard = get_refund_checkpoint_inline_keyboard(refund_id=proposal_index)
            refund_initiated_msg = t(
                "squads_refund_initiated", user_lang, escape_markdown=False, invoice_id=inv_id, amount_usdc=f"{amount_usdc:.2f}", proposal_index=proposal_index
            )
            payloads.append(("sendMessage", build_send_message_payload(chat_id, refund_initiated_msg, parse_mode="Markdown", reply_markup=keyboard)))
        finally:
            conn.close()
        return payloads

    def_label = t("default_item", lang=user_lang, escape_markdown=False)
    parsed = parse_pos_order_input(text, default_item_label=def_label, draft_items=session.get("draft_items"))

    if not parsed["has_price"]:
        session["draft_items"] = parsed["items"]
        session["state"] = "awaiting_price"
        prompt_text = t("price_needed", user_lang, escape_markdown=False, items=parsed["items"])
        payloads.append(
            ("sendMessage", build_send_message_payload(chat_id, prompt_text, parse_mode="Markdown", reply_markup=get_main_reply_keyboard(user_lang)))
        )
        return payloads

    session["state"] = "idle"
    session["draft_items"] = None
    fiat_amt = parsed["amount"]
    fiat_curr = parsed["currency"]
    item_desc = parsed["items"]

    rate_info = get_multitier_fiat_rate(fiat_curr)
    rate = rate_info.get("rate", 1.0)
    usdc_amt = round(fiat_amt / rate, 2)
    inv_id = f"INV-{random.randint(200, 999)}"
    ref_key = generate_secure_reference_key()
    create_invoice_record(
        {"id": inv_id, "reference_pubkey": ref_key, "fiat_currency": fiat_curr, "fiat_amount": fiat_amt, "usdc_amount": usdc_amt}, db_path=db_path
    )

    receipt_text = format_itemized_receipt(inv_id, item_desc, 0.0, usdc_amt, lang=user_lang, fiat_currency=fiat_curr, fiat_amount=fiat_amt, exchange_rate=rate)
    solana_url = generate_solana_pay_url(MERCHANT_WALLET_PUBKEY, usdc_amt, ref_key)
    qr_photo_url = generate_solana_pay_qr_image_url(solana_url, size=300)
    keyboard = get_cancel_invoice_inline_keyboard(inv_id, lang=user_lang)

    photo_payload = generate_telegram_photo_payload(str(chat_id), qr_photo_url, receipt_text, reply_markup=keyboard)
    payloads.append(("sendPhoto", photo_payload))
    return payloads
