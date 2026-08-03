#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Internationalization (i18n) Engine (13 Languages)
Supports 13 world languages covering 85%+ global population with auto-language
detection from Telegram language_code and automatic MarkdownV2 escaping.
Translation data lives in i18n_strings (LANG_META + TRANSLATIONS).
"""

from typing import Dict, Any, Optional
from sanitizer import escape_telegram_markdown_v2
from pos_core.i18n_strings import LANG_META, TRANSLATIONS
from pos_core.i18n_strings_ext import TRANSLATIONS_EXT

TRANSLATIONS = {**TRANSLATIONS, **TRANSLATIONS_EXT}


def get_lang_meta(lang_code: str) -> tuple:
    """Retrieves (flag_emoji, native_name) tuple for a language code."""
    clean = (lang_code or "en").lower().split("-")[0].split("_")[0]
    return LANG_META.get(clean, LANG_META["en"])


def get_localized_confirmation(lang_code: str) -> str:
    """Returns localized language change confirmation message with flag and native language name."""
    flag, name = get_lang_meta(lang_code)
    clean = (lang_code or "en").lower().split("-")[0].split("_")[0]
    lang_dict = TRANSLATIONS.get(clean, TRANSLATIONS["en"])
    template = lang_dict.get("lang_confirm", TRANSLATIONS["en"]["lang_confirm"])
    return template.format(flag=flag, lang_name=name)


def get_localized_message(key: str, lang: Optional[str] = "en", **kwargs: Any) -> str:
    """Backward-compatible alias for t() (escapes MarkdownV2 by default)."""
    return t(key, lang, escape_markdown=True, **kwargs)


def get_main_reply_keyboard(lang: str = "en") -> Dict[str, Any]:
    """Generates localized cashier persistent reply keyboard payload matching active language."""
    clean_lang = (lang or "en").lower().split("-")[0].split("_")[0]
    return {
        "keyboard": [
            [{"text": t("btn_custom", clean_lang, escape_markdown=False)}, {"text": t("btn_quick_uah", clean_lang, escape_markdown=False)}],
            [{"text": t("btn_sales", clean_lang, escape_markdown=False)}, {"text": t("btn_refund", clean_lang, escape_markdown=False)}],
            [{"text": t("btn_lang", clean_lang, escape_markdown=False)}],
        ],
        "resize_keyboard": True,
    }


def get_cancel_invoice_inline_keyboard(invoice_id: str, lang: str = "en") -> Dict[str, Any]:
    """Generates Telegram Inline Keyboard payload for cashier invoice cancellation/voiding."""
    clean_lang = (lang or "en").lower().split("-")[0].split("_")[0]
    btn_label = t("cancel_btn_text", clean_lang, escape_markdown=False)
    return {"inline_keyboard": [[{"text": btn_label, "callback_data": f"cancel_invoice_{invoice_id}"}]]}


def get_refund_checkpoint_inline_keyboard(refund_id: int) -> Dict[str, Any]:
    """Builds inline keyboard presenting Squads v4 refund approve/reject actions for a proposal."""
    return {
        "inline_keyboard": [
            [{"text": "✅ Approve", "callback_data": f"approve_refund_{refund_id}"}, {"text": "🚫 Reject", "callback_data": f"reject_refund_{refund_id}"}]
        ]
    }


def t(key: str, lang: Optional[str] = "en", escape_markdown: bool = True, **kwargs: Any) -> str:
    """
    Retrieves localized message template and formats dynamic variables safely.
    Normalizes regional language codes (e.g. 'pt-BR' -> 'pt', 'es-MX' -> 'es', 'zh-CN' -> 'zh').
    Passes formatted text through escape_telegram_markdown_v2 when escape_markdown=True.
    """
    clean_lang = (lang or "en").lower().split("-")[0].split("_")[0]
    lang_dict = TRANSLATIONS.get(clean_lang, TRANSLATIONS["en"])
    template = lang_dict.get(key, TRANSLATIONS["en"].get(key, key))
    formatted_msg = template.format(**kwargs)

    if escape_markdown:
        return escape_telegram_markdown_v2(formatted_msg)
    return formatted_msg


def format_itemized_receipt(
    invoice_id: str,
    items: str,
    tax_rate_pct: float,
    amount_usdc: float,
    lang: str = "en",
    fiat_currency: Optional[str] = None,
    fiat_amount: Optional[float] = None,
    exchange_rate: Optional[float] = None,
) -> str:
    """
    Formats an itemized POS receipt while safely preserving Telegram MarkdownV2 bold/italic syntax and dual fiat oracle conversion metadata.
    NOTE: The returned string is already fully escaped for MarkdownV2. DO NOT pass the return value to escape_telegram_markdown_v2() to prevent double-escaping bugs!
    """
    tax_amount = round(amount_usdc * (tax_rate_pct / 100.0), 2)
    default_item = t("default_item", lang=lang, escape_markdown=False)

    title_escaped = t("receipt_title", lang=lang, escape_markdown=True, invoice_id=invoice_id)
    tax_escaped = t("receipt_tax", lang=lang, escape_markdown=True, tax_rate_pct=f"{tax_rate_pct:.0f}", tax_amount=f"{tax_amount:.2f}")
    total_escaped = t("receipt_total", lang=lang, escape_markdown=True, amount_usdc=f"{amount_usdc:.2f}")

    raw_items = items if items else default_item
    items_escaped = escape_telegram_markdown_v2(raw_items)
    items_formatted = items_escaped.replace("; ", "\n• ").replace(";", "\n• ")
    if not items_formatted.startswith("• "):
        items_formatted = f"• {items_formatted}"

    fiat_conversion_line = ""
    if fiat_currency and fiat_amount is not None and exchange_rate and exchange_rate > 0:
        clean_fiat_curr = escape_telegram_markdown_v2(str(fiat_currency))
        clean_fiat_amt = escape_telegram_markdown_v2(f"{fiat_amount:.2f}")
        clean_rate = escape_telegram_markdown_v2(f"{exchange_rate:.2f}")
        fiat_conversion_line = rf"• Charged: {clean_fiat_amt} {clean_fiat_curr} \(Rate: {clean_rate}\)" + "\n"

    return (
        f"*{title_escaped}*\n"
        f"───────────────────────────\n"
        f"{items_formatted}\n"
        f"───────────────────────────\n"
        f"• {tax_escaped}\n"
        f"{fiat_conversion_line}"
        f"• *{total_escaped}*"
    )
