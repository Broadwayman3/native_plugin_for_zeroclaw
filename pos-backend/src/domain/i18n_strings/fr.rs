use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert("payment_success", "✅ Paiement Confirmé !\nFacture #{invoice_id}\nMontant : {amount} {currency}\nTx : {tx_sig}");
    m.insert("payment_pending", "⏳ En Attente de Paiement...\nFacture #{invoice_id}\nMontant : {amount} {currency}\nLien : {pay_url}\n📱 Scannez avec Phantom, Solflare ou tout portefeuille Solana");
    m.insert(
        "refund_initiated",
        "🔄 Remboursement Demandé !\nFacture #{invoice_id}\nIndice : {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Erreur de Remboursement : {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Erreur : Devise non prise en charge '{currency}'",
    );
    m.insert("receipt_title", "☕ Reçu ZeroClaw POS #{invoice_id}");
    m.insert("receipt_tax", "Taxe ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "TOTAL: ${amount_usdc} USDC");
    m.insert("default_item", "Commande Standard");
    m.insert(
        "wallet_hint",
        "📱 Scannez avec Phantom, Solflare ou tout portefeuille Solana",
    );
    m.insert(
        "lang_confirm",
        "🌐 Langue de l'interface modifiée avec succès en {flag} {lang_name} !",
    );
    m.insert("welcome", "☕ *Bienvenue sur le terminal POS ZeroClaw Solana !*\n\nSélectionnez une action ou saisissez un montant :");
    m.insert("custom_help", "✍️ *Saisissez le montant et la devise dans votre message :*\n\nExemples :\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert("price_needed", "✍️ Veuillez préciser le prix total et la devise pour '{items}'\n\nExemple :\n• `{items} 500 UAH`");
    m.insert(
        "select_lang",
        "🌐 *Sélectionnez la langue de l'interface :*",
    );
    m.insert("btn_custom", "✍️ Entrer un montant");
    m.insert("btn_quick_uah", "☕ Reçu rapide ({amount} {currency})");
    m.insert("btn_sales", "📊 Résumé des ventes");
    m.insert("btn_refund", "🔄 Remboursement");
    m.insert("btn_lang", "🌐 Langues (13)");
    m.insert("btn_approve", "✅ Approuver");
    m.insert("btn_reject", "🚫 Rejeter");
    m.insert("cancel_btn_text", "❌ Annuler la facture / Void");
    m.insert("void_confirmed", "❌ Facture #{invoice_id} annulée !");
    m.insert(
        "refund_approved",
        "✅ Proposition de remboursement créée dans Squads v4 !\n• Facture : #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ La facture #{invoice_id} est déjà annulée ou a été payée.",
    );
    m.insert("unauthorized_approve", "⛔ Non autorisé : seul le gérant du magasin peut approuver les propositions de remboursement Squads v4.");
    m.insert(
        "squads_refund_approved",
        "✅ Proposition de remboursement Squads v4 #{proposal_index} approuvée !",
    );
    m.insert("unauthorized_reject", "⛔ Non autorisé : seul le gérant du magasin peut rejeter les propositions de remboursement Squads v4.");
    m.insert("squads_refund_rejected", "🚫 Proposition de remboursement Squads v4 #{proposal_index} rejetée. Facture restaurée à 'paid'.");
    m.insert(
        "refund_prompt",
        "♻️ Veuillez saisir l'ID de la facture à rembourser (ex. : INV-101) :",
    );
    m.insert("squads_refund_initiated", "🏛️ *Proposition Multisig Squads v4 initiée*\n───────────────────────────\n• Facture : `{invoice_id}`\n• Montant : *{amount_usdc} USDC*\n• Indice de proposition : `#{proposal_index}` (En attente On-Chain)\n\nApprouver la proposition de remboursement Squads v4 ?");
    all.insert("fr", m);
}
