#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Internationalization (i18n) Engine (13 Languages)
Supports 13 world languages covering 85%+ global population with auto-language
detection from Telegram language_code and automatic MarkdownV2 escaping.
"""

from typing import Dict, Any, Optional
from sanitizer import escape_telegram_markdown_v2

LANG_META: Dict[str, tuple] = {
    "uk": ("🇺🇦", "Українська"),
    "en": ("🇺🇸", "English"),
    "pt": ("🇧🇷", "Português"),
    "es": ("🇪🇸", "Español"),
    "de": ("🇩🇪", "Deutsch"),
    "fr": ("🇫🇷", "Français"),
    "it": ("🇮🇹", "Italiano"),
    "pl": ("🇵🇱", "Polski"),
    "tr": ("🇹🇷", "Türkçe"),
    "ja": ("🇯🇵", "日本語"),
    "zh": ("🇨🇳", "中文"),
    "ar": ("🇸🇦", "العربية"),
    "hi": ("🇮🇳", "हिन्दी")
}

TRANSLATIONS: Dict[str, Dict[str, str]] = {
    "en": {
        "payment_success": "✅ Payment Confirmed!\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Awaiting Payment...\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nPay URL: {pay_url}\n📱 Scan with Phantom, Solflare or any Solana Wallet",
        "refund_initiated": "🔄 Refund Requested!\nInvoice #{invoice_id}\nProposal Index: {proposal_idx}",
        "refund_error": "⚠️ Refund Error: {error_msg}",
        "unsupported_currency": "❌ Error: Unsupported fiat currency '{currency}'",
        "receipt_title": "☕ ZeroClaw POS Receipt #{invoice_id}",
        "receipt_tax": "Tax ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "TOTAL: ${amount_usdc} USDC",
        "default_item": "Standard Order",
        "wallet_hint": "📱 Scan with Phantom, Solflare or any Solana Wallet",
        "lang_confirm": "🌐 Interface language successfully changed to {flag} {lang_name}!",
        "welcome": "☕ *Welcome to ZeroClaw Solana POS Terminal!*\n\nSelect an action on the keyboard below or type custom amount (e.g. `150 UAH`, `35.5 BRL`, `12 USD`):",
        "custom_help": "✍️ *Enter amount and currency in your message:*\n\nExamples:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Select interface language / Оберіть мову:*",
        "btn_custom": "✍️ Enter custom amount",
        "btn_quick_uah": "☕ Quick receipt (200 UAH)",
        "btn_sales": "📊 Sales Summary",
        "btn_refund": "🔄 Refund",
        "btn_lang": "🌐 Languages (13)",
        "cancel_btn_text": "❌ Cancel Invoice / Void",
        "void_confirmed": "❌ Invoice #{invoice_id} voided!",
        "refund_approved": "✅ Refund proposal created in Squads v4!\n• Invoice: #{invoice_id}"
    },
    "uk": {
        "payment_success": "✅ Оплату Підтверджено!\nЧек #{invoice_id}\nСума: {amount} {currency}\nТранзакція: {tx_sig}",
        "payment_pending": "⏳ Очікування Оплати...\nЧек #{invoice_id}\nСума: {amount} {currency}\nПосилання: {pay_url}\n📱 Скануйте через Phantom, Solflare або будь-який гаманець Solana",
        "refund_initiated": "🔄 Ініційовано Рефанд!\nЧек #{invoice_id}\nІндекс пропозиції: {proposal_idx}",
        "refund_error": "⚠️ Помилка Рефанду: {error_msg}",
        "unsupported_currency": "❌ Помилка: Непідтримувана валюта '{currency}'",
        "receipt_title": "☕ ZeroClaw POS Чек #{invoice_id}",
        "receipt_tax": "ПДВ / Податок ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "РАЗОМ: ${amount_usdc} USDC",
        "default_item": "Стандартне Замовлення",
        "wallet_hint": "📱 Скануйте через Phantom, Solflare або будь-який гаманець Solana",
        "lang_confirm": "🌐 Мову інтерфейсу успішно змінено на {flag} {lang_name}!",
        "welcome": "☕ *Вітаємо у ZeroClaw Solana POS Терміналі!*\n\nОберіть дію на клавіатурі внизу або введіть суму текстом (наприклад: `150 UAH`, `35.5 BRL`, `12 USD`):",
        "custom_help": "✍️ *Введіть суму та валюту у повідомленні:*\n\nПриклади:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Оберіть мову інтерфейсу з 13 доступних:*",
        "btn_custom": "✍️ Ввести довільну суму",
        "btn_quick_uah": "☕ Швидкий чек (200 UAH)",
        "btn_sales": "📊 Звіт продажів",
        "btn_refund": "🔄 Рефанд (Refund)",
        "btn_lang": "🌐 13 Мов / Languages",
        "cancel_btn_text": "❌ Скасувати чек / Void",
        "void_confirmed": "❌ Чек #{invoice_id} скасовано!",
        "refund_approved": "✅ Пропозицію повернення коштів створено у Squads v4!\n• Чек: #{invoice_id}"
    },
    "pt": {
        "payment_success": "✅ Pagamento Confirmado!\nFatura #{invoice_id}\nValor: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Aguardando Pagamento...\nFatura #{invoice_id}\nValor: {amount} {currency}\nLink: {pay_url}\n📱 Escaneie com Phantom, Solflare ou qualquer carteira Solana",
        "refund_initiated": "🔄 Reembolso Solicitado!\nFatura #{invoice_id}\nÍndice da Proposta: {proposal_idx}",
        "refund_error": "⚠️ Erro no Reembolso: {error_msg}",
        "unsupported_currency": "❌ Erro: Moeda não suportada '{currency}'",
        "receipt_title": "☕ Recibo ZeroClaw POS #{invoice_id}",
        "receipt_tax": "Imposto ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "TOTAL: ${amount_usdc} USDC",
        "default_item": "Pedido Padrão",
        "wallet_hint": "📱 Escaneie com Phantom, Solflare ou qualquer carteira Solana",
        "lang_confirm": "🌐 Idioma da interface alterado para {flag} {lang_name}!",
        "welcome": "☕ *Bem-vindo ao Terminal POS ZeroClaw Solana!*\n\nSelecione uma ação no teclado abaixo ou digite o valor personalizado:",
        "custom_help": "✍️ *Digite o valor e a moeda na sua mensagem:*\n\nExemplos:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Selecione o idioma da interface:*",
        "btn_custom": "✍️ Digitar valor personalizado",
        "btn_quick_uah": "☕ Recibo rápido (200 UAH)",
        "btn_sales": "📊 Resumo de vendas",
        "btn_refund": "🔄 Reembolso",
        "btn_lang": "🌐 Idiomas (13)",
        "cancel_btn_text": "❌ Cancelar fatura / Void",
        "void_confirmed": "❌ Fatura #{invoice_id} cancelada!",
        "refund_approved": "✅ Proposta de reembolso criada no Squads v4!\n• Fatura: #{invoice_id}"
    },
    "es": {
        "payment_success": "✅ ¡Pago Confirmado!\nFactura #{invoice_id}\nMonto: {amount} {currency}\nFirma: {tx_sig}",
        "payment_pending": "⏳ Esperando Pago...\nFactura #{invoice_id}\nMonto: {amount} {currency}\nEnlace: {pay_url}\n📱 Escanea con Phantom, Solflare o cualquier billetera Solana",
        "refund_initiated": "🔄 ¡Reembolso Solicitado!\nFactura #{invoice_id}\nÍndice: {proposal_idx}",
        "refund_error": "⚠️ Error de Reembolso: {error_msg}",
        "unsupported_currency": "❌ Error: Moneda no soportada '{currency}'",
        "receipt_title": "☕ Recibo ZeroClaw POS #{invoice_id}",
        "receipt_tax": "Impuesto ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "TOTAL: ${amount_usdc} USDC",
        "default_item": "Pedido Estándar",
        "wallet_hint": "📱 Escanea con Phantom, Solflare o cualquier billetera Solana",
        "lang_confirm": "🌐 ¡Idioma de interfaz cambiado a {flag} {lang_name}!",
        "welcome": "☕ *¡Bienvenido al Terminal POS ZeroClaw Solana!*\n\nSeleccione una acción o ingrese el monto:",
        "custom_help": "✍️ *Ingrese el monto y la moneda en su mensaje:*\n\nEjemplos:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Seleccione el idioma de la interfaz:*",
        "btn_custom": "✍️ Ingresar monto personalizado",
        "btn_quick_uah": "☕ Recibo rápido (200 UAH)",
        "btn_sales": "📊 Resumen de ventas",
        "btn_refund": "🔄 Reembolso",
        "btn_lang": "🌐 Idiomas (13)",
        "cancel_btn_text": "❌ Cancelar factura / Void",
        "void_confirmed": "❌ ¡Factura #{invoice_id} cancelada!",
        "refund_approved": "✅ Propuesta de reembolso creada en Squads v4!\n• Factura: #{invoice_id}"
    },
    "de": {
        "payment_success": "✅ Zahlung Bestätigt!\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Warten auf Zahlung...\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nLink: {pay_url}\n📱 Scannen Sie mit Phantom, Solflare oder einer beliebigen Solana-Wallet",
        "refund_initiated": "🔄 Rückerstattung Beantragt!\nRechnung #{invoice_id}\nIndex: {proposal_idx}",
        "refund_error": "⚠️ Rückerstattungsfehler: {error_msg}",
        "unsupported_currency": "❌ Fehler: Nicht unterstützte Währung '{currency}'",
        "receipt_title": "☕ ZeroClaw POS Beleg #{invoice_id}",
        "receipt_tax": "Steuer ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "GESAMT: ${amount_usdc} USDC",
        "default_item": "Standardbestellung",
        "wallet_hint": "📱 Scannen Sie mit Phantom, Solflare oder einer beliebigen Solana-Wallet",
        "lang_confirm": "🌐 Schnittstellensprache erfolgreich geändert auf {flag} {lang_name}!",
        "welcome": "☕ *Willkommen beim ZeroClaw Solana POS Terminal!*\n\nWählen Sie eine Aktion oder geben Sie einen Betrag ein:",
        "custom_help": "✍️ *Geben Sie Betrag und Währung in Ihrer Nachricht ein:*\n\nBeispiele:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Wählen Sie die Schnittstellensprache:*",
        "btn_custom": "✍️ Betrag eingeben",
        "btn_quick_uah": "☕ Schnellbon (200 UAH)",
        "btn_sales": "📊 Verkaufsübersicht",
        "btn_refund": "🔄 Rückerstattung",
        "btn_lang": "🌐 Sprachen (13)",
        "cancel_btn_text": "❌ Beleg stornieren / Void",
        "void_confirmed": "❌ Beleg #{invoice_id} storniert!",
        "refund_approved": "✅ Erstattungsantrag in Squads v4 erstellt!\n• Beleg: #{invoice_id}"
    },
    "fr": {
        "payment_success": "✅ Paiement Confirmé !\nFacture #{invoice_id}\nMontant : {amount} {currency}\nTx : {tx_sig}",
        "payment_pending": "⏳ En Attente de Paiement...\nFacture #{invoice_id}\nMontant : {amount} {currency}\nLien : {pay_url}\n📱 Scannez avec Phantom, Solflare ou tout portefeuille Solana",
        "refund_initiated": "🔄 Remboursement Demandé !\nFacture #{invoice_id}\nIndice : {proposal_idx}",
        "refund_error": "⚠️ Erreur de Remboursement : {error_msg}",
        "unsupported_currency": "❌ Erreur : Devise non prise en charge '{currency}'",
        "receipt_title": "☕ Reçu ZeroClaw POS #{invoice_id}",
        "receipt_tax": "Taxe ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "TOTAL: ${amount_usdc} USDC",
        "default_item": "Commande Standard",
        "wallet_hint": "📱 Scannez avec Phantom, Solflare ou tout portefeuille Solana",
        "lang_confirm": "🌐 Langue de l'interface modifiée avec succès en {flag} {lang_name} !",
        "welcome": "☕ *Bienvenue sur le terminal POS ZeroClaw Solana !*\n\nSélectionnez une action ou saisissez un montant :",
        "custom_help": "✍️ *Saisissez le montant et la devise dans votre message :*\n\nExemples :\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Sélectionnez la langue de l'interface :*",
        "btn_custom": "✍️ Entrer un montant",
        "btn_quick_uah": "☕ Reçu rapide (200 UAH)",
        "btn_sales": "📊 Résumé des ventes",
        "btn_refund": "🔄 Remboursement",
        "btn_lang": "🌐 Langues (13)",
        "cancel_btn_text": "❌ Annuler la facture / Void",
        "void_confirmed": "❌ Facture #{invoice_id} annulée !",
        "refund_approved": "✅ Proposition de remboursement créée dans Squads v4 !\n• Facture : #{invoice_id}"
    },
    "it": {
        "payment_success": "✅ Pagamento Confermato!\nFattura #{invoice_id}\nImporto: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ In Attesa di Pagamento...\nFattura #{invoice_id}\nImporto: {amount} {currency}\nLink: {pay_url}\n📱 Scansiona con Phantom, Solflare o qualsiasi portafoglio Solana",
        "refund_initiated": "🔄 Rimborso Richiesto!\nFattura #{invoice_id}\nIndice: {proposal_idx}",
        "refund_error": "⚠️ Errore di Rimborso: {error_msg}",
        "unsupported_currency": "❌ Errore: Valuta non supportata '{currency}'",
        "receipt_title": "☕ Ricevuta ZeroClaw POS #{invoice_id}",
        "receipt_tax": "Tassa ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "TOTALE: ${amount_usdc} USDC",
        "default_item": "Ordine Standard",
        "wallet_hint": "📱 Scansiona con Phantom, Solflare o qualsiasi portafoglio Solana",
        "lang_confirm": "🌐 Lingua dell'interfaccia modificata con successo in {flag} {lang_name}!",
        "welcome": "☕ *Benvenuto nel Terminale POS ZeroClaw Solana!*\n\nSeleziona un'azione o inserisci l'importo:",
        "custom_help": "✍️ *Inserisci l'importo e la valuta nel messaggio:*\n\nEsempi:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Seleziona la lingua dell'interfaccia:*",
        "btn_custom": "✍️ Inserisci importo",
        "btn_quick_uah": "☕ Scontrino rapido (200 UAH)",
        "btn_sales": "📊 Riepilogo vendite",
        "btn_refund": "🔄 Rimborso",
        "btn_lang": "🌐 Lingue (13)",
        "cancel_btn_text": "❌ Annulla scontrino / Void",
        "void_confirmed": "❌ Scontrino #{invoice_id} annullato!",
        "refund_approved": "✅ Proposta di rimborso creata in Squads v4!\n• Fattura: #{invoice_id}"
    },
    "pl": {
        "payment_success": "✅ Płatność Potwierdzona!\nFaktura #{invoice_id}\nKwota: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ Oczekiwanie na Płatność...\nFaktura #{invoice_id}\nKwota: {amount} {currency}\nLink: {pay_url}\n📱 Zeskanuj za pomocą Phantom, Solflare lub dowolnego portfela Solana",
        "refund_initiated": "🔄 Żądanie Zwrotu!\nFaktura #{invoice_id}\nIndeks: {proposal_idx}",
        "refund_error": "⚠️ Błąd Zwrotu: {error_msg}",
        "unsupported_currency": "❌ Błąd: Nieobsługiwana waluta '{currency}'",
        "receipt_title": "☕ Paragon ZeroClaw POS #{invoice_id}",
        "receipt_tax": "Podatek ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "SUMA: ${amount_usdc} USDC",
        "default_item": "Zamówienie Standardowe",
        "wallet_hint": "📱 Zeskanuj za pomocą Phantom, Solflare lub dowolnego portfela Solana",
        "lang_confirm": "🌐 Język interfejsu pomyślnie zmieniony na {flag} {lang_name}!",
        "welcome": "☕ *Witaj w terminalu ZeroClaw Solana POS!*\n\nWybierz akcję lub wpisz kwotę:",
        "custom_help": "✍️ *Wpisz kwotę i walutę w wiadomości:*\n\nPrzykłady:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Wybierz język interfejsu:*",
        "btn_custom": "✍️ Wpisz kwotę",
        "btn_quick_uah": "☕ Szybki paragon (200 UAH)",
        "btn_sales": "📊 Podsumowanie sprzedaży",
        "btn_refund": "🔄 Zwrot",
        "btn_lang": "🌐 Języki (13)",
        "cancel_btn_text": "❌ Anuluj paragon / Void",
        "void_confirmed": "❌ Paragon #{invoice_id} anulowany!",
        "refund_approved": "✅ Wniosek o zwrot utworzony w Squads v4!\n• Paragon: #{invoice_id}"
    },
    "tr": {
        "payment_success": "✅ Ödeme Onaylandı!\nFatura #{invoice_id}\nTutar: {amount} {currency}\nİşlem: {tx_sig}",
        "payment_pending": "⏳ Ödeme Bekleniyor...\nFatura #{invoice_id}\nTutar: {amount} {currency}\nBağlantı: {pay_url}\n📱 Phantom, Solflare veya herhangi bir Solana Cüzdanı ile tarayın",
        "refund_initiated": "🔄 İade İstendi!\nFatura #{invoice_id}\nDizin: {proposal_idx}",
        "refund_error": "⚠️ İade Hatası: {error_msg}",
        "unsupported_currency": "❌ Hata: Desteklenmeyen para birimi '{currency}'",
        "receipt_title": "☕ ZeroClaw POS Fişi #{invoice_id}",
        "receipt_tax": "Vergi ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "TOPLAM: ${amount_usdc} USDC",
        "default_item": "Standart Sipariş",
        "wallet_hint": "📱 Phantom, Solflare veya herhangi bir Solana Cüzdanı ile tarayın",
        "lang_confirm": "🌐 Arayüz dili başarıyla {flag} {lang_name} olarak değiştirildi!",
        "welcome": "☕ *ZeroClaw Solana POS Terminaline Hoş Geldiniz!*\n\nBir işlem seçin veya tutar girin:",
        "custom_help": "✍️ *Mesajınızda tutarı ve para birimini girin:*\n\nÖrnekler:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *Arayüz dilini seçin:*",
        "btn_custom": "✍️ Özel tutar girin",
        "btn_quick_uah": "☕ Hızlı fiş (200 UAH)",
        "btn_sales": "📊 Satış Özeti",
        "btn_refund": "🔄 İade",
        "btn_lang": "🌐 Diller (13)",
        "cancel_btn_text": "❌ Fişi İptal Et / Void",
        "void_confirmed": "❌ Fiş #{invoice_id} iptal edildi!",
        "refund_approved": "✅ Squads v4 iade teklifi oluşturuldu!\n• Fiş: #{invoice_id}"
    },
    "ja": {
        "payment_success": "✅ 支払い完了!\n請求書 #{invoice_id}\n金額: {amount} {currency}\nTx: {tx_sig}",
        "payment_pending": "⏳ 支払い待ち...\n請求書 #{invoice_id}\n金額: {amount} {currency}\nリンク: {pay_url}\n📱 Phantom、Solflare、または任意のSolanaウォレットでスキャン",
        "refund_initiated": "🔄 返金要求!\n請求書 #{invoice_id}\nインデックス: {proposal_idx}",
        "refund_error": "⚠️ 返金エラー: {error_msg}",
        "unsupported_currency": "❌ エラー: 未対応の通貨 '{currency}'",
        "receipt_title": "☕ ZeroClaw POS レシート #{invoice_id}",
        "receipt_tax": "税 ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "合計: ${amount_usdc} USDC",
        "default_item": "標準注文",
        "wallet_hint": "📱 Phantom、Solflare、または任意のSolanaウォレットでスキャン",
        "lang_confirm": "🌐 インターフェース言語が {flag} {lang_name} に変更されました！",
        "welcome": "☕ *ZeroClaw Solana POS Terminalへようこそ！*\n\n以下からアクションを選択するか金額を入力してください:",
        "custom_help": "✍️ *メッセージに金額と通貨を入力してください:*\n\n例:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *インターフェース言語を選択:*",
        "btn_custom": "✍️ 金額を入力",
        "btn_quick_uah": "☕ クイックレシート (200 UAH)",
        "btn_sales": "📊 売上概要",
        "btn_refund": "🔄 返金",
        "btn_lang": "🌐 言語 (13)",
        "cancel_btn_text": "❌ レシートをキャンセル / Void",
        "void_confirmed": "❌ レシート #{invoice_id} が取り消されました！",
        "refund_approved": "✅ Squads v4 返金提案が作成されました！\n• レシート: #{invoice_id}"
    },
    "zh": {
        "payment_success": "✅ 支付已确认！\n账单 #{invoice_id}\n金额：{amount} {currency}\n交易：{tx_sig}",
        "payment_pending": "⏳ 等待支付...\n账单 #{invoice_id}\n金额：{amount} {currency}\n链接：{pay_url}\n📱 使用 Phantom、Solflare 或任何 Solana 钱包扫描",
        "refund_initiated": "🔄 已申请退款！\n账单 #{invoice_id}\n索引：{proposal_idx}",
        "refund_error": "⚠️ 退款错误：{error_msg}",
        "unsupported_currency": "❌ 错误：不支持的货币 '{currency}'",
        "receipt_title": "☕ ZeroClaw POS 收据 #{invoice_id}",
        "receipt_tax": "税费 ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "总计: ${amount_usdc} USDC",
        "default_item": "标准订单",
        "wallet_hint": "📱 使用 Phantom、Solflare 或任何 Solana 钱包扫描",
        "lang_confirm": "🌐 界面语言已成功更改为 {flag} {lang_name}！",
        "welcome": "☕ *欢迎使用 ZeroClaw Solana POS 终端！*\n\n请在下方键盘选择操作或输入自定义金额：",
        "custom_help": "✍️ *请在消息中输入金额和货币：*\n\n示例：\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *请选择界面语言：*",
        "btn_custom": "✍️ 输入自定义金额",
        "btn_quick_uah": "☕ 快速收据 (200 UAH)",
        "btn_sales": "📊 销售摘要",
        "btn_refund": "🔄 退款",
        "btn_lang": "🌐 语言 (13)",
        "cancel_btn_text": "❌ 取消收据 / Void",
        "void_confirmed": "❌ 收据 #{invoice_id} 已取消！",
        "refund_approved": "✅ 已在 Squads v4 中创建退款提议！\n• 收据: #{invoice_id}"
    },
    "hi": {
        "payment_success": "✅ भुगतान की पुष्टि की गई!\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलेन-देन: {tx_sig}",
        "payment_pending": "⏳ भुगतान का इंतज़ार...\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलिंक: {pay_url}\n📱 Phantom, Solflare या किसी भी Solana वॉलेट से स्कैन करें",
        "refund_initiated": "🔄 रिफंड का अनुरोध किया गया!\nबीजक #{invoice_id}\nइंडेक्स: {proposal_idx}",
        "refund_error": "⚠️ रिफंड त्रुटि: {error_msg}",
        "unsupported_currency": "❌ त्रुटि: असमर्थित मुद्रा '{currency}'",
        "receipt_title": "☕ ZeroClaw POS रसीद #{invoice_id}",
        "receipt_tax": "कर ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "कुल: ${amount_usdc} USDC",
        "default_item": "मानक ऑर्डर",
        "wallet_hint": "📱 Phantom, Solflare या किसी भी Solana वॉलेट से स्कैन करें",
        "lang_confirm": "🌐 इंटरफ़ेस भाषा सफलतापूर्वक {flag} {lang_name} में बदल दी गई!",
        "welcome": "☕ *ZeroClaw Solana POS टर्मिनल में आपका स्वागत है!*\n\nनीचे दिए गए कीबोर्ड पर एक क्रिया चुनें या कस्टम राशि दर्ज करें:",
        "custom_help": "✍️ *अपने संदेश में राशि और मुद्रा दर्ज करें:*\n\nउदाहरण:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *इंटरफ़ेस भाषा चुनें:*",
        "btn_custom": "✍️ कस्टम राशि दर्ज करें",
        "btn_quick_uah": "☕ त्वरित रसीद (200 UAH)",
        "btn_sales": "📊 बिक्री सारांश",
        "btn_refund": "🔄 रिफंड",
        "btn_lang": "🌐 भाषाएँ (13)",
        "cancel_btn_text": "❌ रसीद रद्द करें / Void",
        "void_confirmed": "❌ रसीद #{invoice_id} रद्द कर दी गई!",
        "refund_approved": "✅ Squads v4 में रिफंड प्रस्ताव बनाया गया!\n• रसीद: #{invoice_id}"
    },
    "ar": {
        "payment_success": "✅ تم تأكيد الدفع!\nالفاتورة #{invoice_id}\nالمبلغ: {amount} {currency}\nالمعاملة: {tx_sig}",
        "payment_pending": "⏳ في انتظار الدفع...\nالفاتورة #{invoice_id}\nالمبلغ: {amount} {currency}\nالرابط: {pay_url}\n📱 امسح باستخدام Phantom أو Solflare أو أي محفظة Solana",
        "refund_initiated": "🔄 تم طلب الاسترداد!\nالفاتورة #{invoice_id}\nالفهرس: {proposal_idx}",
        "refund_error": "⚠️ خطأ في الاسترداد: {error_msg}",
        "unsupported_currency": "❌ خطأ: عملة غير مدعومة '{currency}'",
        "receipt_title": "☕ إيصال ZeroClaw POS #{invoice_id}",
        "receipt_tax": "الضريبة ({tax_rate_pct}%): ${tax_amount}",
        "receipt_total": "الإجمالي: ${amount_usdc} USDC",
        "default_item": "طلب قياسي",
        "wallet_hint": "📱 امسح باستخدام Phantom أو Solflare أو أي محفظة Solana",
        "lang_confirm": "🌐 تم تغيير لغة الواجهة بنجاح إلى {flag} {lang_name}!",
        "welcome": "☕ *مرحبًا بك في محطة ZeroClaw Solana POS!*\n\nحدد إجراءً من لوحة المفاتيح أدناه أو أدخل مبلغًا مخصصًا:",
        "custom_help": "✍️ *أدخل المبلغ والعملة في رسالتك:*\n\nأمثلة:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`",
        "select_lang": "🌐 *اختر لغة الواجهة:*",
        "btn_custom": "✍️ إدخال مبلغ مخصص",
        "btn_quick_uah": "☕ إيصال سريع (200 UAH)",
        "btn_sales": "📊 ملخص المبيعات",
        "btn_refund": "🔄 استرداد",
        "btn_lang": "🌐 اللغات (13)",
        "cancel_btn_text": "❌ إلغاء الإيصال / Void",
        "void_confirmed": "❌ تم إلغاء الإيصال #{invoice_id}!",
        "refund_approved": "✅ تم إنشاء اقتراح الاسترداد في Squads v4!\n• الإيصال: #{invoice_id}"
    }
}

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

def get_main_reply_keyboard(lang: str = "en") -> Dict[str, Any]:
    """Generates localized cashier persistent reply keyboard payload matching active language."""
    clean_lang = (lang or "en").lower().split("-")[0].split("_")[0]
    return {
        "keyboard": [
            [
                {"text": t("btn_custom", clean_lang, escape_markdown=False)},
                {"text": t("btn_quick_uah", clean_lang, escape_markdown=False)}
            ],
            [
                {"text": t("btn_sales", clean_lang, escape_markdown=False)},
                {"text": t("btn_refund", clean_lang, escape_markdown=False)}
            ],
            [
                {"text": t("btn_lang", clean_lang, escape_markdown=False)}
            ]
        ],
        "resize_keyboard": True
    }

def get_cancel_invoice_inline_keyboard(invoice_id: str, lang: str = "en") -> Dict[str, Any]:
    """Generates Telegram Inline Keyboard payload for cashier invoice cancellation/voiding."""
    clean_lang = (lang or "en").lower().split("-")[0].split("_")[0]
    btn_label = t("cancel_btn_text", clean_lang, escape_markdown=False)
    return {
        "inline_keyboard": [
            [
                {"text": btn_label, "callback_data": f"cancel_invoice_{invoice_id}"}
            ]
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
    exchange_rate: Optional[float] = None
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

def get_refund_checkpoint_inline_keyboard(proposal_idx: int) -> Dict[str, Any]:
    """Generates Telegram Inline Keyboard payload for human approval checkpoints."""
    return {
        "inline_keyboard": [
            [
                {"text": "✅ Approve Squads v4", "callback_data": f"approve_refund_{proposal_idx}"},
                {"text": "❌ Reject", "callback_data": f"reject_refund_{proposal_idx}"}
            ]
        ]
    }

# Alias for backward compatibility
get_localized_message = t
