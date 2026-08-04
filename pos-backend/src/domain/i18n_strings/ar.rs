use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert("payment_success", "✅ تم تأكيد الدفع!\nالفاتورة #{invoice_id}\nالمبلغ: {amount} {currency}\nالمعاملة: {tx_sig}");
    m.insert("payment_pending", "⏳ في انتظار الدفع...\nالفاتورة #{invoice_id}\nالمبلغ: {amount} {currency}\nالرابط: {pay_url}\n📱 امسح باستخدام Phantom أو Solflare أو أي محفظة Solana");
    m.insert(
        "refund_initiated",
        "🔄 تم طلب الاسترداد!\nالفاتورة #{invoice_id}\nالفهرس: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ خطأ في الاسترداد: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ خطأ: عملة غير مدعومة '{currency}'",
    );
    m.insert("receipt_title", "☕ إيصال ZeroClaw POS #{invoice_id}");
    m.insert("receipt_tax", "الضريبة ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "الإجمالي: ${amount_usdc} USDC");
    m.insert("default_item", "طلب قياسي");
    m.insert(
        "wallet_hint",
        "📱 امسح باستخدام Phantom أو Solflare أو أي محفظة Solana",
    );
    m.insert(
        "lang_confirm",
        "🌐 تم تغيير لغة الواجهة بنجاح إلى {flag} {lang_name}!",
    );
    m.insert("welcome", "☕ *مرحبًا بك في محطة ZeroClaw Solana POS!*\n\nحدد إجراءً من لوحة المفاتيح أدناه أو أدخل مبلغًا مخصصًا:");
    m.insert("custom_help", "✍️ *أدخل المبلغ والعملة في رسالتك:*\n\nأمثلة:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert(
        "price_needed",
        "✍️ يرجى تحديد السعر والعملة لـ '{items}'\n\nمثال:\n• `{items} 500 UAH`",
    );
    m.insert("select_lang", "🌐 *اختر لغة الواجهة:*");
    m.insert("btn_custom", "✍️ إدخال مبلغ مخصص");
    m.insert("btn_quick_uah", "☕ إيصال سريع ({amount} {currency})");
    m.insert("btn_sales", "📊 ملخص المبيعات");
    m.insert("btn_refund", "🔄 استرداد");
    m.insert("btn_lang", "🌐 اللغات (13)");
    m.insert("btn_approve", "✅ موافقة");
    m.insert("btn_reject", "🚫 رفض");
    m.insert("cancel_btn_text", "❌ إلغاء الإيصال / Void");
    m.insert("void_confirmed", "❌ تم إلغاء الإيصال #{invoice_id}!");
    m.insert(
        "refund_approved",
        "✅ تم إنشاء اقتراح الاسترداد في Squads v4!\n• الإيصال: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ الإيصال #{invoice_id} تم إلغاؤه بالفعل أو تم دفعه.",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ غير مصرح: يمكن لمدير المتجر فقط الموافقة على مقترحات استرداد Squads v4.",
    );
    m.insert(
        "squads_refund_approved",
        "✅ تمت الموافقة على اقتراح استرداد Squads v4 #{proposal_index}!",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ غير مصرح: يمكن لمدير المتجر فقط رفض مقترحات استرداد Squads v4.",
    );
    m.insert(
        "squads_refund_rejected",
        "🚫 تم رفض اقتراح استرداد Squads v4 #{proposal_index}. تمت استعادة الإيصال إلى 'paid'.",
    );
    m.insert(
        "refund_prompt",
        "♻️ يرجى إدخال معرف الإيصال لاسترداد المبلغ (مثال: INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *تم بدء اقتراح تعدد التوقيعات Squads v4*\n───────────────────────────\n• الإيصال: `{invoice_id}`\n• المبلغ: *{amount_usdc} USDC*\n• فهرس الاقتراح: `#{proposal_index}` (قيد الانتظار على السلسلة)\n\nهل تريد الموافقة على اقتراح استرداد Squads v4؟");
    all.insert("ar", m);
}
