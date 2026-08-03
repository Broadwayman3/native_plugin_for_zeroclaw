#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Telegram Bot Listener Process
Clean architecture: delegates all domain logic to pos_core.bot_ui, i18n, and db.
Handles only long-polling lifecycle, HTTP transport, and update deduplication.
"""

import os
import sys
import time
import json
import urllib.request
from typing import Dict, Any, Optional

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from pos_core import (
    DB_PATH,
    get_db_connection,
    check_and_register_telegram_update,
    handle_telegram_429_retry,
    LANG_META,
    build_get_updates_payload,
    handle_callback_query,
    handle_text_message,
)

TOKEN = os.getenv("TELEGRAM_BOT_TOKEN") or ""

USER_SESSIONS: Dict[int, Dict[str, Any]] = {}


def get_session(chat_id: int) -> Dict[str, Any]:
    if chat_id not in USER_SESSIONS:
        USER_SESSIONS[chat_id] = {"lang": "uk", "state": "idle", "user_set": False, "draft_items": None}
    return USER_SESSIONS[chat_id]


def tg_request(method: str, payload: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    url = f"https://api.telegram.org/bot{TOKEN}/{method}"
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:  # nosec B310 -- only https://api.telegram.org
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        error_body = e.read().decode("utf-8") if e.fp else "{}"
        try:
            body = json.loads(error_body)
        except json.JSONDecodeError:
            body = {"error_code": e.code}
        status = e.code
        if status == 429:
            retry_after = handle_telegram_429_retry(body)
            print(f"⚠️ TG API Rate Limit (429). Retry after {retry_after}s")
            time.sleep(retry_after)
        elif status in (502, 504):
            print(f"⚠️ TG API Server Error ({status}). Soft retry in 2s")
            time.sleep(2)
        else:
            print(f"⚠️ TG API HTTP Error ({status}): {body.get('description', '')}")
        return None
    except Exception as e:
        print(f"⚠️ TG API Error ({method}):", e)
        return None


def start_polling():
    if not TOKEN:
        raise RuntimeError("TELEGRAM_BOT_TOKEN environment variable not set")
    print("🤖 ZeroClaw POS Bot (Smart Quantity & Multi-Lang Parser) STARTED!")
    offset = 0

    while True:
        try:
            res = tg_request("getUpdates", build_get_updates_payload(offset, 5))
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

                if "callback_query" in update:
                    cb = update["callback_query"]
                    chat_id_cb = cb.get("message", {}).get("chat", {}).get("id", 0)
                    session = get_session(chat_id_cb)
                    payloads = handle_callback_query(cb, session, db_path=DB_PATH)
                    for method, payload in payloads:
                        tg_request(method, payload)
                    continue

                if "message" in update:
                    msg = update["message"]
                    chat_id = msg.get("chat", {}).get("id", 0)
                    session = get_session(chat_id)

                    if not session.get("user_set") and "from" in msg and "language_code" in msg["from"]:
                        tg_lang = msg["from"]["language_code"].lower().split("-")[0]
                        if tg_lang in LANG_META:
                            session["lang"] = tg_lang

                    payloads = handle_text_message(msg, session, db_path=DB_PATH)
                    for method, payload in payloads:
                        tg_request(method, payload)

        except Exception as e:
            print(f"⚠️ Polling cycle error: {e}")
            time.sleep(2)


if __name__ == "__main__":
    start_polling()
