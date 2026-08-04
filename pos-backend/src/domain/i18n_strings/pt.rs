use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ Pagamento Confirmado!\nFatura #{invoice_id}\nValor: {amount} {currency}\nTx: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ Aguardando Pagamento...\nFatura #{invoice_id}\nValor: {amount} {currency}\nLink: {pay_url}\n📱 Escaneie com Phantom, Solflare ou qualquer carteira Solana");
    m.insert(
        "refund_initiated",
        "🔄 Reembolso Solicitado!\nFatura #{invoice_id}\nÍndice da Proposta: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Erro no Reembolso: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Erro: Moeda não suportada '{currency}'",
    );
    m.insert("receipt_title", "☕ Recibo ZeroClaw POS #{invoice_id}");
    m.insert("receipt_tax", "Imposto ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "TOTAL: ${amount_usdc} USDC");
    m.insert("default_item", "Pedido Padrão");
    m.insert(
        "wallet_hint",
        "📱 Escaneie com Phantom, Solflare ou qualquer carteira Solana",
    );
    m.insert(
        "lang_confirm",
        "🌐 Idioma da interface alterado para {flag} {lang_name}!",
    );
    m.insert("welcome", "☕ *Bem-vindo ao Terminal POS ZeroClaw Solana!*\n\nSelecione uma ação no teclado abaixo ou digite o valor personalizado:");
    m.insert("custom_help", "✍️ *Digite o valor e a moeda na sua mensagem:*\n\nExemplos:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert("price_needed", "✍️ Por favor, especifique o preço e a moeda para '{items}'\n\nExemplo:\n• `{items} 500 UAH`");
    m.insert("select_lang", "🌐 *Selecione o idioma da interface:*");
    m.insert("btn_custom", "✍️ Digitar valor personalizado");
    m.insert("btn_quick_uah", "☕ Recibo rápido ({amount} {currency})");
    m.insert("btn_sales", "📊 Resumo de vendas");
    m.insert("btn_refund", "🔄 Reembolso");
    m.insert("btn_lang", "🌐 Idiomas (13)");
    m.insert("btn_approve", "✅ Aprovar");
    m.insert("btn_reject", "🚫 Rejeitar");
    m.insert("cancel_btn_text", "❌ Cancelar fatura / Void");
    m.insert("void_confirmed", "❌ Fatura #{invoice_id} cancelada!");
    m.insert(
        "refund_approved",
        "✅ Proposta de reembolso criada no Squads v4!\n• Fatura: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ A fatura #{invoice_id} já foi cancelada ou paga.",
    );
    m.insert("unauthorized_approve", "⛔ Não autorizado: somente o gerente da loja pode aprovar propostas de reembolso Squads v4.");
    m.insert(
        "squads_refund_approved",
        "✅ Proposta de reembolso Squads v4 #{proposal_index} aprovada!",
    );
    m.insert("unauthorized_reject", "⛔ Não autorizado: somente o gerente da loja pode rejeitar propostas de reembolso Squads v4.");
    m.insert("squads_refund_rejected", "🚫 Proposta de reembolso Squads v4 #{proposal_index} rejeitada. Fatura restaurada para 'paid'.");
    m.insert(
        "refund_prompt",
        "♻️ Digite o ID da fatura para reembolso (ex.: INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Proposta Multisig Squads v4 Iniciada*\n───────────────────────────\n• Fatura: `{invoice_id}`\n• Valor: *{amount_usdc} USDC*\n• Índice da Proposta: `#{proposal_index}` (Pendente On-Chain)\n\nAprovar proposta de reembolso Squads v4?");
    all.insert("pt", m);
}
