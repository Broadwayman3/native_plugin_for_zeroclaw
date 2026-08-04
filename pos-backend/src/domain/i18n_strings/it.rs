use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert("payment_success", "✅ Pagamento Confermato!\nFattura #{invoice_id}\nImporto: {amount} {currency}\nTx: {tx_sig}");
    m.insert("payment_pending", "⏳ In Attesa di Pagamento...\nFattura #{invoice_id}\nImporto: {amount} {currency}\nLink: {pay_url}\n📱 Scansiona con Phantom, Solflare o qualsiasi portafoglio Solana");
    m.insert(
        "refund_initiated",
        "🔄 Rimborso Richiesto!\nFattura #{invoice_id}\nIndice: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Errore di Rimborso: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Errore: Valuta non supportata '{currency}'",
    );
    m.insert("receipt_title", "☕ Ricevuta ZeroClaw POS #{invoice_id}");
    m.insert("receipt_tax", "Tassa ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "TOTALE: ${amount_usdc} USDC");
    m.insert("default_item", "Ordine Standard");
    m.insert(
        "wallet_hint",
        "📱 Scansiona con Phantom, Solflare o qualsiasi portafoglio Solana",
    );
    m.insert(
        "lang_confirm",
        "🌐 Lingua dell'interfaccia modificata con successo in {flag} {lang_name}!",
    );
    m.insert("welcome", "☕ *Benvenuto nel Terminale POS ZeroClaw Solana!*\n\nSeleziona un'azione o inserisci l'importo:");
    m.insert("custom_help", "✍️ *Inserisci l'importo e la valuta nel messaggio:*\n\nEsempi:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert("price_needed", "✍️ Si prega di specificare il prezzo totale e la valuta per '{items}'\n\nEsempio:\n• `{items} 500 UAH`");
    m.insert("select_lang", "🌐 *Seleziona la lingua dell'interfaccia:*");
    m.insert("btn_custom", "✍️ Inserisci importo");
    m.insert("btn_quick_uah", "☕ Scontrino rapido (200 UAH)");
    m.insert("btn_sales", "📊 Riepilogo vendite");
    m.insert("btn_refund", "🔄 Rimborso");
    m.insert("btn_lang", "🌐 Lingue (13)");
    m.insert("btn_approve", "✅ Approva");
    m.insert("btn_reject", "🚫 Rifiuta");
    m.insert("cancel_btn_text", "❌ Annulla scontrino / Void");
    m.insert("void_confirmed", "❌ Scontrino #{invoice_id} annullato!");
    m.insert(
        "refund_approved",
        "✅ Proposta di rimborso creata in Squads v4!\n• Fattura: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ La fattura #{invoice_id} è già stata annullata o pagata.",
    );
    m.insert("unauthorized_approve", "⛔ Non autorizzato: solo il gestore del negozio può approvare le proposte di rimborso Squads v4.");
    m.insert(
        "squads_refund_approved",
        "✅ Proposta di rimborso Squads v4 #{proposal_index} approvata!",
    );
    m.insert("unauthorized_reject", "⛔ Non autorizzato: solo il gestore del negozio può rifiutare le proposte di rimborso Squads v4.");
    m.insert("squads_refund_rejected", "🚫 Proposta di rimborso Squads v4 #{proposal_index} rifiutata. Fattura ripristinata a 'paid'.");
    m.insert(
        "refund_prompt",
        "♻️ Inserisci l'ID della fattura da rimborsare (es.: INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Proposta Multisig Squads v4 Avviata*\n───────────────────────────\n• Fattura: `{invoice_id}`\n• Importo: *{amount_usdc} USDC*\n• Indice proposta: `#{proposal_index}` (In attesa On-Chain)\n\nApprovare la proposta di rimborso Squads v4?");
    all.insert("it", m);
}
