use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ Zahlung Bestätigt!\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nTx: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ Warten auf Zahlung...\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nLink: {pay_url}\n📱 Scannen Sie mit Phantom, Solflare oder einer beliebigen Solana-Wallet");
    m.insert(
        "refund_initiated",
        "🔄 Rückerstattung Beantragt!\nRechnung #{invoice_id}\nIndex: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Rückerstattungsfehler: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Fehler: Nicht unterstützte Währung '{currency}'",
    );
    m.insert("receipt_title", "☕ ZeroClaw POS Beleg #{invoice_id}");
    m.insert("receipt_tax", "Steuer ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "GESAMT: ${amount_usdc} USDC");
    m.insert("default_item", "Standardbestellung");
    m.insert(
        "wallet_hint",
        "📱 Scannen Sie mit Phantom, Solflare oder einer beliebigen Solana-Wallet",
    );
    m.insert(
        "lang_confirm",
        "🌐 Schnittstellensprache erfolgreich geändert auf {flag} {lang_name}!",
    );
    m.insert("welcome", "☕ *Willkommen beim ZeroClaw Solana POS Terminal!*\n\nWählen Sie eine Aktion oder geben Sie einen Betrag ein:");
    m.insert("custom_help", "✍️ *Geben Sie Betrag und Währung in Ihrer Nachricht ein:*\n\nBeispiele:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert("price_needed", "✍️ Bitte geben Sie Gesamtpreis und Währung für '{items}' an\n\nBeispiel:\n• `{items} 500 UAH`");
    m.insert("select_lang", "🌐 *Wählen Sie die Schnittstellensprache:*");
    m.insert("btn_custom", "✍️ Betrag eingeben");
    m.insert("btn_quick_uah", "☕ Schnellbon (200 UAH)");
    m.insert("btn_sales", "📊 Verkaufsübersicht");
    m.insert("btn_refund", "🔄 Rückerstattung");
    m.insert("btn_lang", "🌐 Sprachen (13)");
    m.insert("btn_approve", "✅ Genehmigen");
    m.insert("btn_reject", "🚫 Ablehnen");
    m.insert("cancel_btn_text", "❌ Beleg stornieren / Void");
    m.insert("void_confirmed", "❌ Beleg #{invoice_id} storniert!");
    m.insert(
        "refund_approved",
        "✅ Erstattungsantrag in Squads v4 erstellt!\n• Beleg: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ Rechnung #{invoice_id} wurde bereits storniert oder ist bezahlt.",
    );
    m.insert("unauthorized_approve", "⛔ Nicht autorisiert: Nur der Ladenmanager kann Squads-v4-Rückerstattungsvorschläge genehmigen.");
    m.insert(
        "squads_refund_approved",
        "✅ Squads-v4-Rückerstattungsvorschlag #{proposal_index} genehmigt!",
    );
    m.insert("unauthorized_reject", "⛔ Nicht autorisiert: Nur der Ladenmanager kann Squads-v4-Rückerstattungsvorschläge ablehnen.");
    m.insert("squads_refund_rejected", "🚫 Squads-v4-Rückerstattungsvorschlag #{proposal_index} abgelehnt. Rechnung auf 'paid' zurückgesetzt.");
    m.insert(
        "refund_prompt",
        "♻️ Bitte geben Sie die Rechnungs-ID für die Rückerstattung ein (z. B. INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Squads-v4-Multisig-Vorschlag initiiert*\n───────────────────────────\n• Rechnung: `{invoice_id}`\n• Betrag: *{amount_usdc} USDC*\n• Vorschlagsindex: `#{proposal_index}` (On-Chain ausstehend)\n\nSquads-v4-Rückerstattungsvorschlag genehmigen?");
    all.insert("de", m);
}
