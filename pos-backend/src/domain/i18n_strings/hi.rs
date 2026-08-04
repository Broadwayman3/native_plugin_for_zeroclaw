use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ भुगतान की पुष्टि की गई!\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलेन-देन: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ भुगतान का इंतज़ार...\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलिंक: {pay_url}\n📱 Phantom, Solflare या किसी भी Solana वॉलेट से स्कैन करें");
    m.insert(
        "refund_initiated",
        "🔄 रिफंड का अनुरोध किया गया!\nबीजक #{invoice_id}\nइंडेक्स: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ रिफंड त्रुटि: {error_msg}");
    m.insert("unsupported_currency", "❌ त्रुटि: असमर्थित मुद्रा '{currency}'");
    m.insert("receipt_title", "☕ ZeroClaw POS रसीद #{invoice_id}");
    m.insert("receipt_tax", "कर ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "कुल: ${amount_usdc} USDC");
    m.insert("default_item", "मानक ऑर्डर");
    m.insert(
        "wallet_hint",
        "📱 Phantom, Solflare या किसी भी Solana वॉलेट से स्कैन करें",
    );
    m.insert(
        "lang_confirm",
        "🌐 इंटरफ़ेस भाषा सफलतापूर्वक {flag} {lang_name} में बदल दी गई!",
    );
    m.insert("welcome", "☕ *ZeroClaw Solana POS टर्मिनल में आपका स्वागत है!*\n\nनीचे दिए गए कीबोर्ड पर एक क्रिया चुनें या कस्टम राशि दर्ज करें:");
    m.insert("custom_help", "✍️ *अपने संदेश में राशि और मुद्रा दर्ज करें:*\n\nउदाहरण:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert(
        "price_needed",
        "✍️ कृपया '{items}' के लिए कुल मूल्य और मुद्रा निर्दिष्ट करें\n\nउदाहरण:\n• `{items} 500 UAH`",
    );
    m.insert("select_lang", "🌐 *इंटरफ़ेस भाषा चुनें:*");
    m.insert("btn_custom", "✍️ कस्टम राशि दर्ज करें");
    m.insert("btn_quick_uah", "☕ त्वरित रसीद (200 UAH)");
    m.insert("btn_sales", "📊 बिक्री सारांश");
    m.insert("btn_refund", "🔄 रिफंड");
    m.insert("btn_lang", "🌐 भाषाएँ (13)");
    m.insert("btn_approve", "✅ मंजूरी");
    m.insert("btn_reject", "🚫 अस्वीकृति");
    m.insert("cancel_btn_text", "❌ रसीद रद्द करें / Void");
    m.insert("void_confirmed", "❌ रसीद #{invoice_id} रद्द कर दी गई!");
    m.insert(
        "refund_approved",
        "✅ Squads v4 में रिफंड प्रस्ताव बनाया गया!\n• रसीद: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ रसीद #{invoice_id} पहले ही रद्द कर दी गई है या उसका भुगतान हो चुका है।",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ अनधिकृत: केवल स्टोर प्रबंधक ही Squads v4 रिफंड प्रस्तावों को अनुमोदित कर सकता है।",
    );
    m.insert(
        "squads_refund_approved",
        "✅ Squads v4 रिफंड प्रस्ताव #{proposal_index} अनुमोदित किया गया!",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ अनधिकृत: केवल स्टोर प्रबंधक ही Squads v4 रिफंड प्रस्तावों को अस्वीकार कर सकता है।",
    );
    m.insert("squads_refund_rejected", "🚫 Squads v4 रिफंड प्रस्ताव #{proposal_index} अस्वीकार कर दिया गया। रसीद 'paid' पर बहाल कर दी गई।");
    m.insert(
        "refund_prompt",
        "♻️ कृपया रिफंड के लिए रसीद ID दर्ज करें (जैसे, INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Squads v4 मल्टीसिग प्रस्ताव शुरू किया गया*\n───────────────────────────\n• रसीद: `{invoice_id}`\n• राशि: *{amount_usdc} USDC*\n• प्रस्ताव इंडेक्स: `#{proposal_index}` (ऑन-चेन लंबित)\n\nSquads v4 रिफंड प्रस्ताव अनुमोदित करें?");
    all.insert("hi", m);
}
