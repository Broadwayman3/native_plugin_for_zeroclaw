use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
    let mut m = HashMap::new();
    m.insert(
        "payment_success",
        "✅ ¡Pago Confirmado!\nFactura #{invoice_id}\nMonto: {amount} {currency}\nFirma: {tx_sig}",
    );
    m.insert("payment_pending", "⏳ Esperando Pago...\nFactura #{invoice_id}\nMonto: {amount} {currency}\nEnlace: {pay_url}\n📱 Escanea con Phantom, Solflare o cualquier billetera Solana");
    m.insert(
        "refund_initiated",
        "🔄 ¡Reembolso Solicitado!\nFactura #{invoice_id}\nÍndice: {proposal_idx}",
    );
    m.insert("refund_error", "⚠️ Error de Reembolso: {error_msg}");
    m.insert(
        "unsupported_currency",
        "❌ Error: Moneda no soportada '{currency}'",
    );
    m.insert("receipt_title", "☕ Recibo ZeroClaw POS #{invoice_id}");
    m.insert("receipt_tax", "Impuesto ({tax_rate_pct}%): ${tax_amount}");
    m.insert("receipt_total", "TOTAL: ${amount_usdc} USDC");
    m.insert("default_item", "Pedido Estándar");
    m.insert(
        "wallet_hint",
        "📱 Escanea con Phantom, Solflare o cualquier billetera Solana",
    );
    m.insert(
        "lang_confirm",
        "🌐 ¡Idioma de interfaz cambiado a {flag} {lang_name}!",
    );
    m.insert("welcome", "☕ *¡Bienvenido al Terminal POS ZeroClaw Solana!*\n\nSeleccione una acción o ingrese el monto:");
    m.insert("custom_help", "✍️ *Ingrese el monto y la moneda en su mensaje:*\n\nEjemplos:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
    m.insert("price_needed", "✍️ Por favor especifique el precio y la moneda para '{items}'\n\nEjemplo:\n• `{items} 500 UAH`");
    m.insert("select_lang", "🌐 *Seleccione el idioma de la interfaz:*");
    m.insert("btn_custom", "✍️ Ingresar monto personalizado");
    m.insert("btn_quick_uah", "☕ Recibo rápido (200 UAH)");
    m.insert("btn_sales", "📊 Resumen de ventas");
    m.insert("btn_refund", "🔄 Reembolso");
    m.insert("btn_lang", "🌐 Idiomas (13)");
    m.insert("btn_approve", "✅ Aprobar");
    m.insert("btn_reject", "🚫 Rechazar");
    m.insert("cancel_btn_text", "❌ Cancelar factura / Void");
    m.insert("void_confirmed", "❌ ¡Factura #{invoice_id} cancelada!");
    m.insert(
        "refund_approved",
        "✅ Propuesta de reembolso creada en Squads v4!\n• Factura: #{invoice_id}",
    );
    m.insert(
        "invoice_already_cancelled",
        "⚠️ La factura #{invoice_id} ya está cancelada o ha sido pagada.",
    );
    m.insert("unauthorized_approve", "⛔ No autorizado: solo el gerente de la tienda puede aprobar propuestas de reembolso de Squads v4.");
    m.insert(
        "squads_refund_approved",
        "✅ ¡Propuesta de reembolso Squads v4 #{proposal_index} aprobada!",
    );
    m.insert("unauthorized_reject", "⛔ No autorizado: solo el gerente de la tienda puede rechazar propuestas de reembolso de Squads v4.");
    m.insert("squads_refund_rejected", "🚫 Propuesta de reembolso Squads v4 #{proposal_index} rechazada. Factura restaurada a 'paid'.");
    m.insert(
        "refund_prompt",
        "♻️ Ingrese el ID de la factura a reembolsar (ej.: INV-101):",
    );
    m.insert("squads_refund_initiated", "🏛️ *Propuesta Multisig de Squads v4 Iniciada*\n───────────────────────────\n• Factura: `{invoice_id}`\n• Monto: *{amount_usdc} USDC*\n• Índice de Propuesta: `#{proposal_index}` (Pendiente On-Chain)\n\n¿Aprobar propuesta de reembolso Squads v4?");
    all.insert("es", m);
}
