use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ Ödeme Onaylandı!\nFatura #{invoice_id}\nTutar: {amount} {currency}\nİşlem: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ Ödeme Bekleniyor...\nFatura #{invoice_id}\nTutar: {amount} {currency}\nBağlantı: {pay_url}\n📱 Phantom, Solflare veya herhangi bir Solana Cüzdanı ile tarayın");
    m.insert(
        "refund_initiated",
        "🔄 İade İstendi!\nFatura #{invoice_id}\nDizin: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ İade Hatası: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Hata: Desteklenmeyen para birimi '{currency}'",
    );
    m.insert("receipt_title", "☕ ZeroClaw POS Fişi #{invoice_id}");
    m.insert("receipt_tax", "Vergi ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "TOPLAM: ${amount_usdc} USDC");
    m.insert("default_item", "Standart Sipariş");
    m.insert(
        "wallet_hint",
        "📱 Phantom, Solflare veya herhangi bir Solana Cüzdanı ile tarayın",
    );
    m.insert(
        "lang_confirm",
        "🌐 Arayüz dili başarıyla {flag} {lang_name} olarak değiştirildi!",
    );
    m.insert(
        "welcome",
        "☕ *ZeroClaw Solana POS Terminaline Hoş Geldiniz!*\n\nBir işlem seçin veya tutar girin:",
    );
    m.insert("custom_help", "✍️ *Mesajınızda tutarı ve para birimini girin:*\n\nÖrnekler:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert("price_needed", "✍️ Lütfen '{items}' için toplam fiyatı ve para birimini belirtin\n\nÖrnek:\n• `{items} 500 UAH`");
    m.insert("select_lang", "🌐 *Arayüz dilini seçin:*");
    m.insert("btn_custom", "✍️ Özel tutar girin");
    m.insert("btn_quick_uah", "☕ Hızlı fiş ({amount} {currency})");
    m.insert("btn_sales", "📊 Satış Özeti");
    m.insert("btn_refund", "🔄 İade");
    m.insert("btn_lang", "🌐 Diller (13)");
    m.insert("btn_approve", "✅ Onayla");
    m.insert("btn_reject", "🚫 Reddet");
    m.insert("cancel_btn_text", "❌ Fişi İptal Et / Void");
    m.insert("void_confirmed", "❌ Fiş #{invoice_id} iptal edildi!");
    m.insert(
        "refund_approved",
        "✅ Squads v4 iade teklifi oluşturuldu!\n• Fiş: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ #{invoice_id} numaralı fatura zaten iptal edilmiş veya ödenmiştir.",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ Yetkisiz: Squads v4 iade tekliflerini yalnızca mağaza yöneticisi onaylayabilir.",
    );
    m.insert(
        "squads_refund_approved",
        "✅ Squads v4 iade teklifi #{proposal_index} onaylandı!",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ Yetkisiz: Squads v4 iade tekliflerini yalnızca mağaza yöneticisi reddedebilir.",
    );
    m.insert("squads_refund_rejected", "🚫 Squads v4 iade teklifi #{proposal_index} reddedildi. Fatura 'paid' durumuna geri alındı.");
    m.insert(
        "refund_prompt",
        "♻️ İade edilecek fatura kimliğini girin (ör. INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Squads v4 Multisig Teklifi Başlatıldı*\n───────────────────────────\n• Fatura: `{invoice_id}`\n• Tutar: *{amount_usdc} USDC*\n• Teklif Dizini: `#{proposal_index}` (On-Chain Bekliyor)\n\nSquads v4 iade teklifi onaylansın mı?");
    all.insert("tr", m);
}
