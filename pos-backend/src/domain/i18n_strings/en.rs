use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ Payment Confirmed!\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nTx: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ Awaiting Payment...\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nPay URL: {pay_url}\n📱 Scan with Phantom, Solflare or any Solana Wallet");
    m.insert(
        "refund_initiated",
        "🔄 Refund Requested!\nInvoice #{invoice_id}\nProposal Index: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Refund Error: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Error: Unsupported fiat currency '{currency}'",
    );
    m.insert("receipt_title", "☕ ZeroClaw POS Receipt #{invoice_id}");
    m.insert("receipt_tax", "Tax ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "TOTAL: ${amount_usdc} USDC");
    m.insert("default_item", "Standard Order");
    m.insert(
        "wallet_hint",
        "📱 Scan with Phantom, Solflare or any Solana Wallet",
    );
    m.insert(
        "lang_confirm",
        "🌐 Interface language successfully changed to {flag} {lang_name}!",
    );
    m.insert("welcome", "☕ *Welcome to ZeroClaw Solana POS Terminal!*\n\nSelect an action on the keyboard below or type custom amount (e.g. `150 UAH`, `35.5 BRL`, `12 USD`):");
    m.insert("custom_help", "✍️ *Enter amount and currency in your message:*\n\nExamples:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert(
        "price_needed",
        "✍️ Please specify total price and currency for '{items}'\n\nExample:\n• `{items} 500 UAH`",
    );
    m.insert(
        "select_lang",
        "🌐 *Select interface language / Оберіть мову:*",
    );
    m.insert("btn_custom", "✍️ Enter custom amount");
    m.insert("btn_quick_uah", "☕ Quick receipt ({amount} {currency})");
    m.insert("btn_sales", "📊 Sales Summary");
    m.insert("btn_refund", "🔄 Refund");
    m.insert("btn_lang", "🌐 Languages (13)");
    m.insert("btn_approve", "✅ Approve");
    m.insert("btn_reject", "🚫 Reject");
    m.insert("cancel_btn_text", "❌ Cancel Invoice / Void");
    m.insert("void_confirmed", "❌ Invoice #{invoice_id} voided!");
    m.insert(
        "refund_approved",
        "✅ Refund proposal created in Squads v4!\n• Invoice: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ Invoice #{invoice_id} is already cancelled or has been paid.",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ Unauthorized: Only the store manager can approve Squads v4 refund proposals.",
    );
    m.insert(
        "squads_refund_approved",
        "✅ Squads v4 Refund Proposal #{proposal_index} approved!",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ Unauthorized: Only the store manager can reject Squads v4 refund proposals.",
    );
    m.insert("squads_refund_rejected", "🚫 Squads v4 Refund proposal #{proposal_index} has been rejected. Invoice restored to 'paid'.");
    m.insert(
        "refund_prompt",
        "♻️ Please enter the invoice ID to refund (e.g., INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Squads v4 Multisig Proposal Initiated*\n───────────────────────────\n• Invoice: `{invoice_id}`\n• Amount: *{amount_usdc} USDC*\n• Proposal Index: `#{proposal_index}` (On-Chain Pending)\n\nApprove Squads v4 refund proposal?");
    all.insert("en", m);
}
