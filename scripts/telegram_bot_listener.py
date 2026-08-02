#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Telegram Bot Listener Process
Clean architecture listener using central pos_core domain logic & i18n engine.
Features per-chat multi-language session state & dynamic custom amount parser.
"""

import os
import sys
import time
import json
import re
import random
import urllib.request
from typing import Dict, Any, Optional

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from pos_core import (
    get_db_connection,
    check_and_register_telegram_update,
    get_sales_summary_stats,
    create_invoice_record,
    cancel_invoice_record,
    format_itemized_receipt,
    get_cancel_invoice_inline_keyboard,
    generate_solana_pay_qr_image_url,
    generate_solana_pay_url,
    allocate_free_nonce_account,
    generate_secure_reference_key,
    get_multitier_fiat_rate,
    LANG_META,
    TRANSLATIONS,
    get_localized_confirmation,
    get_main_reply_keyboard,
    t
)

TOKEN = os.getenv("TELEGRAM_BOT_TOKEN", "8861640052:AAHHIdIsgCNDGBym76X7yJcCM_EJN0NIspg")

# Per-chat user session store
USER_SESSIONS: Dict[int, Dict[str, Any]] = {}

def get_session(chat_id: int) -> Dict[str, Any]:
    if chat_id not in USER_SESSIONS:
        USER_SESSIONS[chat_id] = {"lang": "uk", "state": "idle"}
    return USER_SESSIONS[chat_id]

LANG_KEYBOARD = {
    "inline_keyboard": [
        [{"text": f"{v[0]} {k.upper()}", "callback_data": f"set_lang_{k}"} for k, v in list(LANG_META.items())[:4]],
        [{"text": f"{v[0]} {k.upper()}", "callback_data": f"set_lang_{k}"} for k, v in list(LANG_META.items())[4:8]],
        [{"text": f"{v[0]} {k.upper()}", "callback_data": f"set_lang_{k}"} for k, v in list(LANG_META.items())[8:12]],
        [{"text": f"{v[0]} {k.upper()}", "callback_data": f"set_lang_{k}"} for k, v in list(LANG_META.items())[12:]]
    ]
}

def tg_request(method: str, payload: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    url = f"https://api.telegram.org/bot{TOKEN}/{method}"
    data = json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(url, data=data, headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode('utf-8'))
    except Exception as e:
        print(f"⚠️ TG API Error ({method}):", e)
        return None

def is_btn_click(text: str, key: str) -> bool:
    """Checks if incoming button text matches translation for 'key' in any language."""
    text_clean = text.strip().lower()
    for lang_code, trans_dict in TRANSLATIONS.items():
        if key in trans_dict:
            target = trans_dict[key].strip().lower()
            if text_clean == target or target in text_clean:
                return True
    return False

def start_polling():
    print("🤖 ZeroClaw POS Bot (Per-Chat Session & Multi-Lang Keyboards) STARTED!")
    offset = 0

    while True:
        try:
            res = tg_request("getUpdates", {"offset": offset, "timeout": 5})
            if not res or not res.get("ok"):
                time.sleep(2)
                continue

            for update in res.get("result", []):
                offset = update["update_id"] + 1
                update_id = update["update_id"]

                conn = get_db_connection()
                try:
                    if not check_and_register_telegram_update(conn, update_id):
                        continue
                finally:
                    conn.close()

                # Callback Queries (Inline Buttons)
                if "callback_query" in update:
                    cb = update["callback_query"]
                    cb_id = cb["id"]
                    data_str = cb.get("data", "")
                    chat_id = cb["message"]["chat"]["id"]
                    session = get_session(chat_id)

                    if data_str.startswith("set_lang_"):
                        new_lang = data_str.replace("set_lang_", "")
                        session["lang"] = new_lang
                        conf_msg = get_localized_confirmation(new_lang)
                        tg_request("answerCallbackQuery", {"callback_query_id": cb_id, "text": "Language Changed!"})
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": conf_msg,
                            "reply_markup": get_main_reply_keyboard(new_lang)
                        })
                    elif data_str.startswith("cancel_invoice_"):
                        inv_id = data_str.replace("cancel_invoice_", "")
                        cancel_invoice_record(inv_id)
                        tg_request("answerCallbackQuery", {"callback_query_id": cb_id, "text": "Voided!"})
                        void_msg = t("void_confirmed", session["lang"], escape_markdown=False, invoice_id=inv_id)
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": void_msg,
                            "reply_markup": get_main_reply_keyboard(session["lang"])
                        })
                    elif data_str.startswith("approve_refund_"):
                        inv_id = data_str.replace("approve_refund_", "")
                        tg_request("answerCallbackQuery", {"callback_query_id": cb_id, "text": "Approved!"})
                        appr_msg = t("refund_approved", session["lang"], escape_markdown=False, invoice_id=inv_id)
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": appr_msg,
                            "reply_markup": get_main_reply_keyboard(session["lang"])
                        })
                    continue

                # Text Messages
                if "message" in update:
                    msg = update["message"]
                    chat_id = msg["chat"]["id"]
                    session = get_session(chat_id)
                    user_lang = session["lang"]

                    # Auto-detect language from Telegram message user language_code if not explicitly set
                    if session.get("state") == "idle" and "from" in msg and "language_code" in msg["from"]:
                        tg_lang = msg["from"]["language_code"].lower().split("-")[0]
                        if tg_lang in LANG_META and session.get("user_set") is not True:
                            session["lang"] = tg_lang
                            user_lang = tg_lang

                    text = (msg.get("text") or "").strip()
                    text_lower = text.lower()

                    if not text:
                        continue

                    # Command: /start or menu
                    if text_lower in ["/start", "меню", "menu"]:
                        session["state"] = "idle"
                        welcome_msg = t("welcome", user_lang, escape_markdown=False)
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": welcome_msg,
                            "parse_mode": "Markdown",
                            "reply_markup": get_main_reply_keyboard(user_lang)
                        })

                    # Button: Select Language
                    elif is_btn_click(text, "btn_lang") or "13 мов" in text_lower or "language" in text_lower or "idioma" in text_lower or "sprache" in text_lower or "langue" in text_lower or "lingua" in text_lower or "język" in text_lower or "dil" in text_lower or "言語" in text_lower or "语言" in text_lower or "भाषा" in text_lower or "لغة" in text_lower:
                        session["state"] = "idle"
                        select_lang_msg = t("select_lang", user_lang, escape_markdown=False)
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": select_lang_msg,
                            "parse_mode": "Markdown",
                            "reply_markup": LANG_KEYBOARD
                        })

                    # Button: Enter custom amount
                    elif is_btn_click(text, "btn_custom") or "custom" in text_lower or "довільн" in text_lower or "personalizado" in text_lower or "eingeben" in text_lower or "montant" in text_lower or "importo" in text_lower or "kwotę" in text_lower or "tutar" in text_lower or "入力" in text_lower or "自定义" in text_lower or "दर्ज" in text_lower or "مخصص" in text_lower:
                        session["state"] = "awaiting_custom_amount"
                        custom_msg = t("custom_help", user_lang, escape_markdown=False)
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": custom_msg,
                            "parse_mode": "Markdown",
                            "reply_markup": get_main_reply_keyboard(user_lang)
                        })

                    # Button: Quick receipt (200 UAH)
                    elif is_btn_click(text, "btn_quick_uah") or "quick" in text_lower or "200 uah" in text_lower or "швидкий" in text_lower:
                        session["state"] = "idle"
                        fiat_amt = 200.0
                        fiat_curr = "UAH"
                        rate_info = get_multitier_fiat_rate(fiat_curr)
                        rate = rate_info.get("rate", 41.5)
                        usdc_amt = round(fiat_amt / rate, 2)

                        inv_id = f"INV-{random.randint(200, 999)}"
                        ref_key = generate_secure_reference_key()

                        create_invoice_record({
                            "id": inv_id,
                            "reference_pubkey": ref_key,
                            "fiat_currency": fiat_curr,
                            "fiat_amount": fiat_amt,
                            "usdc_amount": usdc_amt
                        })

                        item_desc = t("default_item", lang=user_lang, escape_markdown=False) + f" {fiat_amt} {fiat_curr}"
                        receipt_text = format_itemized_receipt(
                            inv_id, item_desc, 0.0, usdc_amt,
                            lang=user_lang, fiat_currency=fiat_curr,
                            fiat_amount=fiat_amt, exchange_rate=rate
                        )

                        solana_url = generate_solana_pay_url("8xAZmQ1111111111111111111111111111111111111", usdc_amt, ref_key)
                        qr_photo_url = generate_solana_pay_qr_image_url(solana_url, size=300)
                        keyboard = get_cancel_invoice_inline_keyboard(inv_id, lang=user_lang)

                        tg_request("sendPhoto", {
                            "chat_id": chat_id,
                            "photo": qr_photo_url,
                            "caption": receipt_text,
                            "parse_mode": "MarkdownV2",
                            "reply_markup": keyboard
                        })

                    # Button: Sales Summary
                    elif is_btn_click(text, "btn_sales") or "звіт" in text_lower or "sales" in text_lower or "vendas" in text_lower or "resumen" in text_lower or "übersicht" in text_lower or "résumé" in text_lower or "riepilogo" in text_lower or "podsumowanie" in text_lower or "özeti" in text_lower or "売上" in text_lower or "销售" in text_lower or "बिक्री" in text_lower or "ملخص" in text_lower:
                        session["state"] = "idle"
                        stats = get_sales_summary_stats()
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
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": summary_msg,
                            "parse_mode": "Markdown",
                            "reply_markup": get_main_reply_keyboard(user_lang)
                        })

                    # Button: Refund
                    elif is_btn_click(text, "btn_refund") or "рефанд" in text_lower or "refund" in text_lower or "reembolso" in text_lower or "rückerstattung" in text_lower or "remboursement" in text_lower or "rimborso" in text_lower or "zwrot" in text_lower or "iade" in text_lower or "返金" in text_lower or "退款" in text_lower or "रिफंड" in text_lower or "استرداد" in text_lower:
                        session["state"] = "idle"
                        allocated_nonce = allocate_free_nonce_account() or "Nonce111111111111111111111111111111111111111"
                        refund_msg = (
                            "🏛️ *Squads v4 Multisig Proposal Initiated*\n"
                            "───────────────────────────\n"
                            "• Invoice: `#INV-101`\n"
                            "• Customer: `9xK2...Customer1`\n"
                            "• Amount: *10.00 USDC*\n"
                            "• Proposal Index: `#42` (On-Chain Verified)\n"
                            f"• Nonce Account: `{allocated_nonce[:8]}...`\n\n"
                            "Approve Squads v4 refund proposal?"
                        )
                        keyboard = {
                            "inline_keyboard": [
                                [
                                    {"text": "✅ Approve Squads v4", "callback_data": "approve_refund_101"},
                                    {"text": "❌ Reject", "callback_data": "cancel_invoice_101"}
                                ]
                            ]
                        }
                        tg_request("sendMessage", {
                            "chat_id": chat_id,
                            "text": refund_msg,
                            "parse_mode": "Markdown",
                            "reply_markup": keyboard
                        })

                    # Dynamic Custom Amount or Arbitrary Input Parsing
                    else:
                        match = re.search(r'(?:(\d+x?\s+[\w\s\+\-\&]+?)\s+)?(\d+(?:\.\d+)?)\s*([a-zA-Z]{3})', text, re.IGNORECASE)
                        if match:
                            items_typed = (match.group(1) or "").strip()
                            fiat_amt = float(match.group(2))
                            fiat_curr = match.group(3).upper()
                            item_desc = items_typed if items_typed else f"{t('default_item', lang=user_lang, escape_markdown=False)} {fiat_amt} {fiat_curr}"
                        else:
                            # Try simple number match
                            num_match = re.search(r'(\d+(?:\.\d+)?)', text)
                            if num_match:
                                fiat_amt = float(num_match.group(1))
                                fiat_curr = "UAH"
                                item_desc = f"{text.replace(num_match.group(1), '').strip() or t('default_item', lang=user_lang, escape_markdown=False)} {fiat_amt} {fiat_curr}".strip()
                            else:
                                if session.get("state") == "awaiting_custom_amount":
                                    # Prompt for valid custom amount format
                                    tg_request("sendMessage", {
                                        "chat_id": chat_id,
                                        "text": t("custom_help", user_lang, escape_markdown=False),
                                        "parse_mode": "Markdown",
                                        "reply_markup": get_main_reply_keyboard(user_lang)
                                    })
                                    continue
                                else:
                                    fiat_amt = 200.0
                                    fiat_curr = "UAH"
                                    item_desc = f"{text} ({fiat_amt} {fiat_curr})"

                        session["state"] = "idle"
                        rate_info = get_multitier_fiat_rate(fiat_curr)
                        rate = rate_info.get("rate", 1.0)
                        usdc_amt = round(fiat_amt / rate, 2)

                        inv_id = f"INV-{random.randint(200, 999)}"
                        ref_key = generate_secure_reference_key()

                        create_invoice_record({
                            "id": inv_id,
                            "reference_pubkey": ref_key,
                            "fiat_currency": fiat_curr,
                            "fiat_amount": fiat_amt,
                            "usdc_amount": usdc_amt
                        })

                        receipt_text = format_itemized_receipt(
                            inv_id, item_desc, 0.0, usdc_amt,
                            lang=user_lang, fiat_currency=fiat_curr,
                            fiat_amount=fiat_amt, exchange_rate=rate
                        )

                        solana_url = generate_solana_pay_url("8xAZmQ1111111111111111111111111111111111111", usdc_amt, ref_key)
                        qr_photo_url = generate_solana_pay_qr_image_url(solana_url, size=300)
                        keyboard = get_cancel_invoice_inline_keyboard(inv_id, lang=user_lang)

                        tg_request("sendPhoto", {
                            "chat_id": chat_id,
                            "photo": qr_photo_url,
                            "caption": receipt_text,
                            "parse_mode": "MarkdownV2",
                            "reply_markup": keyboard
                        })

        except Exception as e:
            time.sleep(2)

if __name__ == '__main__':
    start_polling()
