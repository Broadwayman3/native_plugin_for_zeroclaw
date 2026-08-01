#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Internationalization (i18n) Engine (13 Languages)
Supports 13 world languages covering 85%+ global population with auto-language
detection from Telegram language_code and automatic MarkdownV2 escaping.
"""

from typing import Dict, Any, Optional
from sanitizer import escape_telegram_markdown_v2

TRANSLATIONS: Dict[str, Dict[str, str]] = {
    "en": {
        "payment_success": "✅ Payment Confirmed!\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Awaiting Payment...\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nPay URL: {pay_url}",
        "refund_initiated": "🔄 Refund Requested!\nInvoice #{invoice_id}\nProposal Index: {proposal_idx}",
        "refund_error": "⚠️ Refund Error: {error_msg}",
        "unsupported_currency": "❌ Error: Unsupported fiat currency '{currency}'"
    },
    "uk": {
        "payment_success": "✅ Оплату Підтверджено!\nЧек #{invoice_id}\nСума: {amount} {currency}\nТранзакція: {tx_sig}",
        "payment_pending": "⏳ Очікування Оплати...\nЧек #{invoice_id}\nСума: {amount} {currency}\nПосилання: {pay_url}",
        "refund_initiated": "🔄 Ініційовано Рефанд!\nЧек #{invoice_id}\nІндекс пропозиції: {proposal_idx}",
        "refund_error": "⚠️ Помилка Рефанду: {error_msg}",
        "unsupported_currency": "❌ Помилка: Непідтримувана валюта '{currency}'"
    },
    "pt": {
        "payment_success": "✅ Pagamento Confirmado!\nFatura #{invoice_id}\nValor: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Aguardando Pagamento...\nFatura #{invoice_id}\nValor: {amount} {currency}\nLink: {pay_url}",
        "refund_initiated": "🔄 Reembolso Solicitado!\nFatura #{invoice_id}\nÍndice da Proposta: {proposal_idx}",
        "refund_error": "⚠️ Erro no Reembolso: {error_msg}",
        "unsupported_currency": "❌ Erro: Moeda não suportada '{currency}'"
    },
    "es": {
        "payment_success": "✅ ¡Pago Confirmado!\nFactura #{invoice_id}\nMonto: {amount} {currency}\nFirma: {tx_sig}",
        "payment_pending": "⏳ Esperando Pago...\nFactura #{invoice_id}\nMonto: {amount} {currency}\nEnlace: {pay_url}",
        "refund_initiated": "🔄 ¡Reembolso Solicitado!\nFactura #{invoice_id}\nÍndice: {proposal_idx}",
        "refund_error": "⚠️ Error de Reembolso: {error_msg}",
        "unsupported_currency": "❌ Error: Moneda no soportada '{currency}'"
    },
    "de": {
        "payment_success": "✅ Zahlung Bestätigt!\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Warten auf Zahlung...\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nLink: {pay_url}",
        "refund_initiated": "🔄 Rückerstattung Beantragt!\nRechnung #{invoice_id}\nIndex: {proposal_idx}",
        "refund_error": "⚠️ Rückerstattungsfehler: {error_msg}",
        "unsupported_currency": "❌ Fehler: Nicht unterstützte Währung '{currency}'"
    },
    "fr": {
        "payment_success": "✅ Paiement Confirmé !\nFacture #{invoice_id}\nMontant : {amount} {currency}\nTx : {tx_sig}",
        "payment_pending": "⏳ En Attente de Paiement...\nFacture #{invoice_id}\nMontant : {amount} {currency}\nLien : {pay_url}",
        "refund_initiated": "🔄 Remboursement Demandé !\nFacture #{invoice_id}\nIndice : {proposal_idx}",
        "refund_error": "⚠️ Erreur de Remboursement : {error_msg}",
        "unsupported_currency": "❌ Erreur : Devise non prise en charge '{currency}'"
    },
    "it": {
        "payment_success": "✅ Pagamento Confermato!\nFattura #{invoice_id}\nImporto: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ In Attesa di Pagamento...\nFattura #{invoice_id}\nImporto: {amount} {currency}\nLink: {pay_url}",
        "refund_initiated": "🔄 Rimborso Richiesto!\nFattura #{invoice_id}\nIndice: {proposal_idx}",
        "refund_error": "⚠️ Errore di Rimborso: {error_msg}",
        "unsupported_currency": "❌ Errore: Valuta non supportata '{currency}'"
    },
    "pl": {
        "payment_success": "✅ Płatność Potwierdzona!\nFaktura #{invoice_id}\nKwota: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Oczekiwanie na Płatność...\nFaktura #{invoice_id}\nKwota: {amount} {currency}\nLink: {pay_url}",
        "refund_initiated": "🔄 Żądanie Zwrotu!\nFaktura #{invoice_id}\nIndeks: {proposal_idx}",
        "refund_error": "⚠️ Błąd Zwrotu: {error_msg}",
        "unsupported_currency": "❌ Błąd: Nieobsługiwana waluta '{currency}'"
    },
    "tr": {
        "payment_success": "✅ Ödeme Onaylandı!\nFatura #{invoice_id}\nTutar: {amount} {currency}\nİşlem: {tx_sig}",
        "payment_pending": "⏳ Ödeme Bekleniyor...\nFatura #{invoice_id}\nTutar: {amount} {currency}\nBağlantı: {pay_url}",
        "refund_initiated": "🔄 İade İstendi!\nFatura #{invoice_id}\nDizin: {proposal_idx}",
        "refund_error": "⚠️ İade Hatası: {error_msg}",
        "unsupported_currency": "❌ Hata: Desteklenmeyen para birimi '{currency}'"
    },
    "ja": {
        "payment_success": "✅ 支払い完了!\n請求書 #{invoice_id}\n金額: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ 支払い待ち...\n請求書 #{invoice_id}\n金額: {amount} {currency}\nリンク: {pay_url}",
        "refund_initiated": "🔄 返金要求!\n請求書 #{invoice_id}\nインデックス: {proposal_idx}",
        "refund_error": "⚠️ 返金エラー: {error_msg}",
        "unsupported_currency": "❌ エラー: 未対応の通貨 '{currency}'"
    },
    "zh": {
        "payment_success": "✅ 支付已确认！\n账单 #{invoice_id}\n金额：{amount} {currency}\n交易：{tx_sig}",
        "payment_pending": "⏳ 等待支付...\n账单 #{invoice_id}\n金额：{amount} {currency}\n链接：{pay_url}",
        "refund_initiated": "🔄 已申请退款！\n账单 #{invoice_id}\n索引：{proposal_idx}",
        "refund_error": "⚠️ 退款错误：{error_msg}",
        "unsupported_currency": "❌ 错误：不支持的货币 '{currency}'"
    },
    "hi": {
        "payment_success": "✅ भुगतान की पुष्टि की गई!\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलेन-देन: {tx_sig}",
        "payment_pending": "⏳ भुगतान का इंतज़ार...\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलिंक: {pay_url}",
        "refund_initiated": "🔄 रिफंड का अनुरोध किया गया!\nबीजक #{invoice_id}\nइंडेक्स: {proposal_idx}",
        "refund_error": "⚠️ रिफंड त्रुटि: {error_msg}",
        "unsupported_currency": "❌ त्रुटि: असमर्थित मुद्रा '{currency}'"
    },
    "ar": {
        "payment_success": "✅ تم تأكيد الدفع!\nالفاتورة #{invoice_id}\nالمبلغ: {amount} {currency}\nالمعاملة: {tx_sig}",
        "payment_pending": "⏳ في انتظار الدفع...\nالفاتورة #{invoice_id}\nالمبلغ: {amount} {currency}\nالرابط: {pay_url}",
        "refund_initiated": "🔄 تم طلب الاسترداد!\nالفاتورة #{invoice_id}\nالفهرس: {proposal_idx}",
        "refund_error": "⚠️ خطأ في الاسترداد: {error_msg}",
        "unsupported_currency": "❌ خطأ: عملة غير مدعومة '{currency}'"
    }
}

def t(key: str, lang: Optional[str] = "en", escape_markdown: bool = True, **kwargs: Any) -> str:
    """
    Retrieves localized message template and formats dynamic variables safely.
    Normalizes regional language codes (e.g. 'pt-BR' -> 'pt', 'es-MX' -> 'es', 'zh-CN' -> 'zh').
    Passes full formatted text through escape_telegram_markdown_v2 when escape_markdown=True.
    """
    clean_lang = (lang or "en").lower().split("-")[0].split("_")[0]
    lang_dict = TRANSLATIONS.get(clean_lang, TRANSLATIONS["en"])
    template = lang_dict.get(key, TRANSLATIONS["en"].get(key, key))
    formatted_msg = template.format(**kwargs)
    
    if escape_markdown:
        return escape_telegram_markdown_v2(formatted_msg)
    return formatted_msg

# Alias for backward compatibility
get_localized_message = t
