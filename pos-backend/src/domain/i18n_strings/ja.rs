use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ 支払い完了!\n請求書 #{invoice_id}\n金額: {amount} {currency}\nTx: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ 支払い待ち...\n請求書 #{invoice_id}\n金額: {amount} {currency}\nリンク: {pay_url}\n📱 Phantom、Solflare、または任意のSolanaウォレットでスキャン");
    m.insert(
        "refund_initiated",
        "🔄 返金要求!\n請求書 #{invoice_id}\nインデックス: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ 返金エラー: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ エラー: 未対応の通貨 '{currency}'",
    );
    m.insert("receipt_title", "☕ ZeroClaw POS レシート #{invoice_id}");
    m.insert("receipt_tax", "税 ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "合計: ${amount_usdc} USDC");
    m.insert("default_item", "標準注文");
    m.insert(
        "wallet_hint",
        "📱 Phantom、Solflare、または任意のSolanaウォレットでスキャン",
    );
    m.insert(
        "lang_confirm",
        "🌐 インターフェース言語が {flag} {lang_name} に変更されました！",
    );
    m.insert("welcome", "☕ *ZeroClaw Solana POS Terminalへようこそ！*\n\n以下からアクションを選択するか金額を入力してください:");
    m.insert("custom_help", "✍️ *メッセージに金額と通貨を入力してください:*\n\n例:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert(
        "price_needed",
        "✍️ '{items}' の合計価格と通貨を指定してください\n\n例:\n• `{items} 500 UAH`",
    );
    m.insert("select_lang", "🌐 *インターフェース言語を選択:*");
    m.insert("btn_custom", "✍️ 金額を入力");
    m.insert("btn_quick_uah", "☕ クイックレシート (200 UAH)");
    m.insert("btn_sales", "📊 売上概要");
    m.insert("btn_refund", "🔄 返金");
    m.insert("btn_lang", "🌐 言語 (13)");
    m.insert("btn_approve", "✅ 承認");
    m.insert("btn_reject", "🚫 拒否");
    m.insert("cancel_btn_text", "❌ レシートをキャンセル / Void");
    m.insert(
        "void_confirmed",
        "❌ レシート #{invoice_id} が取り消されました！",
    );
    m.insert(
        "refund_approved",
        "✅ Squads v4 返金提案が作成されました！\n• レシート: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ 請求書 #{invoice_id} はすでにキャンセルされているか、支払い済みです。",
    );
    m.insert(
        "unauthorized_approve",
        "⛔ 未承認: Squads v4 の返金提案を承認できるのは店舗マネージャーのみです。",
    );
    m.insert(
        "squads_refund_approved",
        "✅ Squads v4 返金提案 #{proposal_index} が承認されました！",
    );
    m.insert(
        "unauthorized_reject",
        "⛔ 未承認: Squads v4 の返金提案を拒否できるのは店舗マネージャーのみです。",
    );
    m.insert("squads_refund_rejected", "🚫 Squads v4 返金提案 #{proposal_index} が拒否されました。請求書は 'paid' に復元されました。");
    m.insert(
        "refund_prompt",
        "♻️ 返金する請求書IDを入力してください (例: INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Squads v4 マルチシグ提案を開始*\n───────────────────────────\n• 請求書: `{invoice_id}`\n• 金額: *{amount_usdc} USDC*\n• 提案インデックス: `#{proposal_index}` (オンチェーン保留中)\n\nSquads v4 返金提案を承認しますか?");
    all.insert("ja", m);
}
