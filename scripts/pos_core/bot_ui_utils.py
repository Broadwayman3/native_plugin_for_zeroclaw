#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Bot UI Utilities
Pure utility functions: keyboards, button matching, order parsing, payload builders, rate lookup.
Zero external dependencies. All functions are side-effect-free payload builders.
"""

import os
import re
import sys
from typing import Dict, Any, Optional

SCRIPT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from pos_core.i18n import LANG_META, TRANSLATIONS

MANAGER_TELEGRAM_ID = int(os.getenv("MANAGER_TELEGRAM_ID", "0"))
MERCHANT_WALLET_PUBKEY = os.getenv("MERCHANT_WALLET_PUBKEY", "8xAZmQ1111111111111111111111111111111111111")


def generate_lang_inline_keyboard() -> Dict[str, Any]:
    items = list(LANG_META.items())
    rows = []
    for i in range(0, len(items), 4):
        chunk = items[i : i + 4]
        rows.append([{"text": f"{v[0]} {k.upper()}", "callback_data": f"set_lang_{k}"} for k, v in chunk])
    return {"inline_keyboard": rows}


def is_btn_click(text: str, key: str) -> bool:
    text_clean = text.strip().lower()
    for lang_code, trans_dict in TRANSLATIONS.items():
        if key in trans_dict:
            target = trans_dict[key].strip().lower()
            if text_clean == target or target in text_clean:
                return True
    return False


def parse_pos_order_input(text: str, default_item_label: str = "Standard Order", draft_items: Optional[str] = None) -> Dict[str, Any]:
    text_clean = text.strip()
    m_curr = re.search(r"(\d+(?:\.\d+)?)\s*([a-zA-Z]{3}|₴|\$|€|R\$|zł|TL)\b", text_clean, re.IGNORECASE)
    if m_curr:
        amt = float(m_curr.group(1))
        curr = m_curr.group(2).upper()
        if curr == "₴":
            curr = "UAH"
        elif curr == "$":
            curr = "USD"
        elif curr == "€":
            curr = "EUR"
        elif curr in ["R$", "REAL"]:
            curr = "BRL"
        elif curr == "ZŁ":
            curr = "PLN"
        matched_str = m_curr.group(0)
        items_part = text_clean.replace(matched_str, "").strip()
        if items_part:
            final_item = items_part
        elif draft_items:
            final_item = draft_items
        else:
            final_item = f"{default_item_label} {amt} {curr}"
        return {"has_price": True, "items": final_item, "amount": amt, "currency": curr}
    m_num = re.search(r"^\s*(\d+(?:\.\d+)?)\s*$", text_clean)
    if m_num:
        amt = float(m_num.group(1))
        curr = "UAH"
        final_item = draft_items if draft_items else f"{default_item_label} {amt} {curr}"
        return {"has_price": True, "items": final_item, "amount": amt, "currency": curr}
    return {"has_price": False, "items": text_clean, "amount": None, "currency": None}


def build_send_message_payload(chat_id: int, text: str, parse_mode: Optional[str] = None, reply_markup: Optional[Dict] = None) -> Dict[str, Any]:
    payload: Dict[str, Any] = {"chat_id": chat_id, "text": text}
    if parse_mode:
        payload["parse_mode"] = parse_mode
    if reply_markup:
        payload["reply_markup"] = reply_markup
    return payload


def build_answer_callback_payload(callback_query_id: str, text: str, show_alert: bool = False) -> Dict[str, Any]:
    payload: Dict[str, Any] = {"callback_query_id": callback_query_id, "text": text}
    if show_alert:
        payload["show_alert"] = True
    return payload


def build_get_updates_payload(offset: int, timeout: int = 5) -> Dict[str, Any]:
    return {"offset": offset, "timeout": timeout}
