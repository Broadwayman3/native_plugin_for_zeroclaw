use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert("payment_success", "✅ Płatność Potwierdzona!\nFaktura #{invoice_id}\nKwota: {amount} {currency}\nTx: {tx_sig}");
    m.insert("payment_pending", "⏳ Oczekiwanie na Płatność...\nFaktura #{invoice_id}\nKwota: {amount} {currency}\nLink: {pay_url}\n📱 Zeskanuj za pomocą Phantom, Solflare lub dowolnego portfela Solana");
    m.insert(
        "refund_initiated",
        "🔄 Żądanie Zwrotu!\nFaktura #{invoice_id}\nIndeks: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Błąd Zwrotu: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Błąd: Nieobsługiwana waluta '{currency}'",
    );
    m.insert("receipt_title", "☕ Paragon ZeroClaw POS #{invoice_id}");
    m.insert("receipt_tax", "Podatek ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "SUMA: ${amount_usdc} USDC");
    m.insert("default_item", "Zamówienie Standardowe");
    m.insert(
        "wallet_hint",
        "📱 Zeskanuj za pomocą Phantom, Solflare lub dowolnego portfela Solana",
    );
    m.insert(
        "lang_confirm",
        "🌐 Język interfejsu pomyślnie zmieniony na {flag} {lang_name}!",
    );
    m.insert(
        "welcome",
        "☕ *Witaj w terminalu ZeroClaw Solana POS!*\n\nWybierz akcję lub wpisz kwotę:",
    );
    m.insert("custom_help", "✍️ *Wpisz kwotę i walutę w wiadomości:*\n\nPrzykłady:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert(
        "price_needed",
        "✍️ Podaj łączną cenę i walutę dla '{items}'\n\nPrzykład:\n• `{items} 500 UAH`",
    );
    m.insert("select_lang", "🌐 *Wybierz język interfejsu:*");
    m.insert("btn_custom", "✍️ Wpisz kwotę");
    m.insert("btn_quick_uah", "☕ Szybki paragon ({amount} {currency})");
    m.insert("btn_sales", "📊 Podsumowanie sprzedaży");
    m.insert("btn_refund", "🔄 Zwrot");
    m.insert("btn_lang", "🌐 Języki (13)");
    m.insert("btn_approve", "✅ Zatwierdzić");
    m.insert("btn_reject", "🚫 Odrzucić");
    m.insert("cancel_btn_text", "❌ Anuluj paragon / Void");
    m.insert("void_confirmed", "❌ Paragon #{invoice_id} anulowany!");
    m.insert(
        "refund_approved",
        "✅ Wniosek o zwrot utworzony w Squads v4!\n• Paragon: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ Paragon #{invoice_id} został już anulowany lub opłacony.",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ Nieautoryzowano: tylko menedżer sklepu może zatwierdzać wnioski o zwrot Squads v4.",
    );
    m.insert(
        "squads_refund_approved",
        "✅ Wniosek o zwrot Squads v4 #{proposal_index} zatwierdzony!",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ Nieautoryzowano: tylko menedżer sklepu może odrzucać wnioski o zwrot Squads v4.",
    );
    m.insert(
        "squads_refund_rejected",
        "🚫 Wniosek o zwrot Squads v4 #{proposal_index} odrzucony. Paragon przywrócony do 'paid'.",
    );
    m.insert(
        "refund_prompt",
        "♻️ Podaj ID paragonu do zwrotu (np. INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Zainicjowano wniosek multisig Squads v4*\n───────────────────────────\n• Paragon: `{invoice_id}`\n• Kwota: *{amount_usdc} USDC*\n• Indeks wniosku: `#{proposal_index}` (Oczekuje On-Chain)\n\nZatwierdzić wniosek o zwrot Squads v4?");
    all.insert("pl", m);
}
