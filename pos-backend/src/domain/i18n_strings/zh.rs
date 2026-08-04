use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ 支付已确认！\n账单 #{invoice_id}\n金额：{amount} {currency}\n交易：{tx_sig}",
    );
    m.insert("payment_pending", "⏳ 等待支付...\n账单 #{invoice_id}\n金额：{amount} {currency}\n链接：{pay_url}\n📱 使用 Phantom、Solflare 或任何 Solana 钱包扫描");
    m.insert(
        "refund_initiated",
        "🔄 已申请退款！\n账单 #{invoice_id}\n索引：{proposal_idx}",
    );
    m.insert("refund_error", "⚠️ 退款错误：{error_msg}");
    m.insert("unsupported_currency", "❌ 错误：不支持的货币 '{currency}'");
    m.insert("receipt_title", "☕ ZeroClaw POS 收据 #{invoice_id}");
    m.insert("receipt_tax", "税费 ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "总计: ${amount_usdc} USDC");
    m.insert("default_item", "标准订单");
    m.insert(
        "wallet_hint",
        "📱 使用 Phantom、Solflare 或任何 Solana 钱包扫描",
    );
    m.insert(
        "lang_confirm",
        "🌐 界面语言已成功更改为 {flag} {lang_name}！",
    );
    m.insert(
        "welcome",
        "☕ *欢迎使用 ZeroClaw Solana POS 终端！*\n\n请在下方键盘选择操作或输入自定义金额：",
    );
    m.insert("custom_help", "✍️ *请在消息中输入金额和货币：*\n\n示例：\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert(
        "price_needed",
        "✍️ 请指定 '{items}' 的总价和货币\n\n示例：\n• `{items} 500 UAH`",
    );
    m.insert("select_lang", "🌐 *请选择界面语言：*");
    m.insert("btn_custom", "✍️ 输入自定义金额");
    m.insert("btn_quick_uah", "☕ 快速收据 ({amount} {currency})");
    m.insert("btn_sales", "📊 销售摘要");
    m.insert("btn_refund", "🔄 退款");
    m.insert("btn_lang", "🌐 语言 (13)");
    m.insert("btn_approve", "✅ 批准");
    m.insert("btn_reject", "🚫 拒绝");
    m.insert("cancel_btn_text", "❌ 取消收据 / Void");
    m.insert("void_confirmed", "❌ 收据 #{invoice_id} 已取消！");
    m.insert(
        "refund_approved",
        "✅ 已在 Squads v4 中创建退款提议！\n• 收据: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ 收据 #{invoice_id} 已取消或已付款。",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ 未授权：只有店铺经理可以批准 Squads v4 退款提议。",
    );
    m.insert(
        "squads_refund_approved",
        "✅ Squads v4 退款提议 #{proposal_index} 已批准！",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ 未授权：只有店铺经理可以拒绝 Squads v4 退款提议。",
    );
    m.insert(
        "squads_refund_rejected",
        "🚫 Squads v4 退款提议 #{proposal_index} 已被拒绝。收据已恢复为 'paid'。",
    );
    m.insert(
        "refund_prompt",
        "♻️ 请输入要退款的收据 ID（例如：INV-101）：",
    );
    m.insert("squads_refund_initiated", "🏛️ *Squads v4 多重签名提议已发起*\n───────────────────────────\n• 收据：`{invoice_id}`\n• 金额：*{amount_usdc} USDC*\n• 提议索引：`#{proposal_index}`（链上待处理）\n\n是否批准 Squads v4 退款提议？");
    all.insert("zh", m);
}
