use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};

pub static LANG_META: Lazy<BTreeMap<&'static str, (&'static str, &'static str)>> =
    Lazy::new(|| {
        let mut m = BTreeMap::new();
        m.insert("uk", ("\u{1f1fa}\u{1f1e6}", "Українська"));
        m.insert("en", ("\u{1f1fa}\u{1f1f8}", "English"));
        m.insert("pt", ("\u{1f1e7}\u{1f1f7}", "Português"));
        m.insert("es", ("\u{1f1ea}\u{1f1f8}", "Español"));
        m.insert("de", ("\u{1f1e9}\u{1f1ea}", "Deutsch"));
        m.insert("fr", ("\u{1f1eb}\u{1f1f7}", "Français"));
        m.insert("it", ("\u{1f1ee}\u{1f1f9}", "Italiano"));
        m.insert("pl", ("\u{1f1f5}\u{1f1f1}", "Polski"));
        m.insert("tr", ("\u{1f1f9}\u{1f1f7}", "Türkçe"));
        m.insert("ja", ("\u{1f1ef}\u{1f1f5}", "日本語"));
        m.insert("zh", ("\u{1f1e8}\u{1f1f3}", "中文"));
        m.insert("ar", ("\u{1f1f8}\u{1f1e6}", "العربية"));
        m.insert("hi", ("\u{1f1ee}\u{1f1f3}", "हिन्दी"));
        m
    });

pub static TRANSLATIONS: Lazy<HashMap<&'static str, HashMap<&'static str, &'static str>>> =
    Lazy::new(|| {
        let mut all = HashMap::new();

        // ── en ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ Payment Confirmed!\nInvoice #{invoice_id}\nAmount: {amount} {currency}\nTx: {tx_sig}");
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
            m.insert("price_needed", "✍️ Please specify total price and currency for '{items}'\n\nExample:\n• `{items} 500 UAH`");
            m.insert(
                "select_lang",
                "🌐 *Select interface language / Оберіть мову:*",
            );
            m.insert("btn_custom", "✍️ Enter custom amount");
            m.insert("btn_quick_uah", "☕ Quick receipt (200 UAH)");
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

        // ── uk ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ Оплату Підтверджено!\nЧек #{invoice_id}\nСума: {amount} {currency}\nТранзакція: {tx_sig}");
            m.insert("payment_pending", "⏳ Очікування Оплати...\nЧек #{invoice_id}\nСума: {amount} {currency}\nПосилання: {pay_url}\n📱 Скануйте через Phantom, Solflare або будь-який гаманець Solana");
            m.insert(
                "refund_initiated",
                "🔄 Ініційовано Рефанд!\nЧек #{invoice_id}\nІндекс пропозиції: {proposal_idx}",
            );
            m.insert("refund_error", "⚠️ Помилка Рефанду: {error_msg}");
            m.insert(
                "unsupported_currency",
                "❌ Помилка: Непідтримувана валюта '{currency}'",
            );
            m.insert("receipt_title", "☕ ZeroClaw POS Чек #{invoice_id}");
            m.insert(
                "receipt_tax",
                "ПДВ / Податок ({tax_rate_pct}%): ${tax_amount}",
            );
            m.insert("receipt_total", "РАЗОМ: ${amount_usdc} USDC");
            m.insert("default_item", "Стандартне Замовлення");
            m.insert(
                "wallet_hint",
                "📱 Скануйте через Phantom, Solflare або будь-який гаманець Solana",
            );
            m.insert(
                "lang_confirm",
                "🌐 Мову інтерфейсу успішно змінено на {flag} {lang_name}!",
            );
            m.insert("welcome", "☕ *Вітаємо у ZeroClaw Solana POS Терміналі!*\n\nОберіть дію на клавіатурі внизу або введіть суму текстом (наприклад: `150 UAH`, `35.5 BRL`, `12 USD`):");
            m.insert("custom_help", "✍️ *Введіть суму та валюту у повідомленні:*\n\nПриклади:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
            m.insert("price_needed", "✍️ Будь ласка, введіть загальну суму та валюту для '{items}'\n\nПриклад:\n• `{items} 500 UAH`");
            m.insert(
                "select_lang",
                "🌐 *Оберіть мову інтерфейсу з 13 доступних:*",
            );
            m.insert("btn_custom", "✍️ Ввести довільну суму");
            m.insert("btn_quick_uah", "☕ Швидкий чек (200 UAH)");
            m.insert("btn_sales", "📊 Звіт продажів");
            m.insert("btn_refund", "🔄 Рефанд (Refund)");
            m.insert("btn_lang", "🌐 13 Мов / Languages");
            m.insert("btn_approve", "✅ Схвалити");
            m.insert("btn_reject", "🚫 Відхилити");
            m.insert("cancel_btn_text", "❌ Скасувати чек / Void");
            m.insert("void_confirmed", "❌ Чек #{invoice_id} скасовано!");
            m.insert(
                "refund_approved",
                "✅ Пропозицію повернення коштів створено у Squads v4!\n• Чек: #{invoice_id}",
            );
            m.insert(
                "invoice_already_cancelled",
                "⚠️ Чек #{invoice_id} вже скасовано або його сплачено.",
            );
            m.insert("unauthorized_approve", "⛔ Заборонено: лише менеджер магазину може схвалити пропозицію повернення коштів Squads v4.");
            m.insert(
                "squads_refund_approved",
                "✅ Пропозицію повернення коштів Squads v4 #{proposal_index} схвалено!",
            );
            m.insert("unauthorized_reject", "⛔ Заборонено: лише менеджер магазину може відхилити пропозицію повернення коштів Squads v4.");
            m.insert("squads_refund_rejected", "🚫 Пропозицію повернення коштів Squads v4 #{proposal_index} відхилено. Чек відновлено до статусу 'paid'.");
            m.insert(
                "refund_prompt",
                "♻️ Будь ласка, введіть ID чека для повернення коштів (наприклад, INV-101):",
            );
            m.insert("squads_refund_initiated", "🏛️ *Ініційовано мультипідписну пропозицію Squads v4*\n───────────────────────────\n• Чек: `{invoice_id}`\n• Сума: *{amount_usdc} USDC*\n• Індекс пропозиції: `#{proposal_index}` (Очікування On-Chain)\n\nСхвалити пропозицію повернення коштів Squads v4?");
            all.insert("uk", m);
        }

        // ── pt ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ Pagamento Confirmado!\nFatura #{invoice_id}\nValor: {amount} {currency}\nTx: {tx_sig}");
            m.insert("payment_pending", "⏳ Aguardando Pagamento...\nFatura #{invoice_id}\nValor: {amount} {currency}\nLink: {pay_url}\n📱 Escaneie com Phantom, Solflare ou qualquer carteira Solana");
            m.insert("refund_initiated", "🔄 Reembolso Solicitado!\nFatura #{invoice_id}\nÍndice da Proposta: {proposal_idx}");
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
            m.insert("btn_quick_uah", "☕ Recibo rápido (200 UAH)");
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

        // ── es ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ ¡Pago Confirmado!\nFactura #{invoice_id}\nMonto: {amount} {currency}\nFirma: {tx_sig}");
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

        // ── de ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ Zahlung Bestätigt!\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nTx: {tx_sig}");
            m.insert("payment_pending", "⏳ Warten auf Zahlung...\nRechnung #{invoice_id}\nBetrag: {amount} {currency}\nLink: {pay_url}\n📱 Scannen Sie mit Phantom, Solflare oder einer beliebigen Solana-Wallet");
            m.insert(
                "refund_initiated",
                "🔄 Rückerstattung Beantragt!\nRechnung #{invoice_id}\nIndex: {proposal_idx}",
            );
            m.insert("refund_error", "⚠️ Rückerstattungsfehler: {error_msg}");
            m.insert(
                "unsupported_currency",
                "❌ Fehler: Nicht unterstützte Währung '{currency}'",
            );
            m.insert("receipt_title", "☕ ZeroClaw POS Beleg #{invoice_id}");
            m.insert("receipt_tax", "Steuer ({tax_rate_pct}%): ${tax_amount}");
            m.insert("receipt_total", "GESAMT: ${amount_usdc} USDC");
            m.insert("default_item", "Standardbestellung");
            m.insert(
                "wallet_hint",
                "📱 Scannen Sie mit Phantom, Solflare oder einer beliebigen Solana-Wallet",
            );
            m.insert(
                "lang_confirm",
                "🌐 Schnittstellensprache erfolgreich geändert auf {flag} {lang_name}!",
            );
            m.insert("welcome", "☕ *Willkommen beim ZeroClaw Solana POS Terminal!*\n\nWählen Sie eine Aktion oder geben Sie einen Betrag ein:");
            m.insert("custom_help", "✍️ *Geben Sie Betrag und Währung in Ihrer Nachricht ein:*\n\nBeispiele:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
            m.insert("price_needed", "✍️ Bitte geben Sie Gesamtpreis und Währung für '{items}' an\n\nBeispiel:\n• `{items} 500 UAH`");
            m.insert("select_lang", "🌐 *Wählen Sie die Schnittstellensprache:*");
            m.insert("btn_custom", "✍️ Betrag eingeben");
            m.insert("btn_quick_uah", "☕ Schnellbon (200 UAH)");
            m.insert("btn_sales", "📊 Verkaufsübersicht");
            m.insert("btn_refund", "🔄 Rückerstattung");
            m.insert("btn_lang", "🌐 Sprachen (13)");
            m.insert("btn_approve", "✅ Genehmigen");
            m.insert("btn_reject", "🚫 Ablehnen");
            m.insert("cancel_btn_text", "❌ Beleg stornieren / Void");
            m.insert("void_confirmed", "❌ Beleg #{invoice_id} storniert!");
            m.insert(
                "refund_approved",
                "✅ Erstattungsantrag in Squads v4 erstellt!\n• Beleg: #{invoice_id}",
            );
            m.insert(
                "invoice_already_cancelled",
                "⚠️ Rechnung #{invoice_id} wurde bereits storniert oder ist bezahlt.",
            );
            m.insert("unauthorized_approve", "⛔ Nicht autorisiert: Nur der Ladenmanager kann Squads-v4-Rückerstattungsvorschläge genehmigen.");
            m.insert(
                "squads_refund_approved",
                "✅ Squads-v4-Rückerstattungsvorschlag #{proposal_index} genehmigt!",
            );
            m.insert("unauthorized_reject", "⛔ Nicht autorisiert: Nur der Ladenmanager kann Squads-v4-Rückerstattungsvorschläge ablehnen.");
            m.insert("squads_refund_rejected", "🚫 Squads-v4-Rückerstattungsvorschlag #{proposal_index} abgelehnt. Rechnung auf 'paid' zurückgesetzt.");
            m.insert(
                "refund_prompt",
                "♻️ Bitte geben Sie die Rechnungs-ID für die Rückerstattung ein (z. B. INV-101):",
            );
            m.insert("squads_refund_initiated", "🏛️ *Squads-v4-Multisig-Vorschlag initiiert*\n───────────────────────────\n• Rechnung: `{invoice_id}`\n• Betrag: *{amount_usdc} USDC*\n• Vorschlagsindex: `#{proposal_index}` (On-Chain ausstehend)\n\nSquads-v4-Rückerstattungsvorschlag genehmigen?");
            all.insert("de", m);
        }

        // ── fr ──────────────────────────────────────────────────────────
        {
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
            m.insert("btn_quick_uah", "☕ Reçu rapide (200 UAH)");
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

        // ── it ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ Pagamento Confermato!\nFattura #{invoice_id}\nImporto: {amount} {currency}\nTx: {tx_sig}");
            m.insert("payment_pending", "⏳ In Attesa di Pagamento...\nFattura #{invoice_id}\nImporto: {amount} {currency}\nLink: {pay_url}\n📱 Scansiona con Phantom, Solflare o qualsiasi portafoglio Solana");
            m.insert(
                "refund_initiated",
                "🔄 Rimborso Richiesto!\nFattura #{invoice_id}\nIndice: {proposal_idx}",
            );
            m.insert("refund_error", "⚠️ Errore di Rimborso: {error_msg}");
            m.insert(
                "unsupported_currency",
                "❌ Errore: Valuta non supportata '{currency}'",
            );
            m.insert("receipt_title", "☕ Ricevuta ZeroClaw POS #{invoice_id}");
            m.insert("receipt_tax", "Tassa ({tax_rate_pct}%): ${tax_amount}");
            m.insert("receipt_total", "TOTALE: ${amount_usdc} USDC");
            m.insert("default_item", "Ordine Standard");
            m.insert(
                "wallet_hint",
                "📱 Scansiona con Phantom, Solflare o qualsiasi portafoglio Solana",
            );
            m.insert(
                "lang_confirm",
                "🌐 Lingua dell'interfaccia modificata con successo in {flag} {lang_name}!",
            );
            m.insert("welcome", "☕ *Benvenuto nel Terminale POS ZeroClaw Solana!*\n\nSeleziona un'azione o inserisci l'importo:");
            m.insert("custom_help", "✍️ *Inserisci l'importo e la valuta nel messaggio:*\n\nEsempi:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
            m.insert("price_needed", "✍️ Si prega di specificare il prezzo totale e la valuta per '{items}'\n\nEsempio:\n• `{items} 500 UAH`");
            m.insert("select_lang", "🌐 *Seleziona la lingua dell'interfaccia:*");
            m.insert("btn_custom", "✍️ Inserisci importo");
            m.insert("btn_quick_uah", "☕ Scontrino rapido (200 UAH)");
            m.insert("btn_sales", "📊 Riepilogo vendite");
            m.insert("btn_refund", "🔄 Rimborso");
            m.insert("btn_lang", "🌐 Lingue (13)");
            m.insert("btn_approve", "✅ Approva");
            m.insert("btn_reject", "🚫 Rifiuta");
            m.insert("cancel_btn_text", "❌ Annulla scontrino / Void");
            m.insert("void_confirmed", "❌ Scontrino #{invoice_id} annullato!");
            m.insert(
                "refund_approved",
                "✅ Proposta di rimborso creata in Squads v4!\n• Fattura: #{invoice_id}",
            );
            m.insert(
                "invoice_already_cancelled",
                "⚠️ La fattura #{invoice_id} è già stata annullata o pagata.",
            );
            m.insert("unauthorized_approve", "⛔ Non autorizzato: solo il gestore del negozio può approvare le proposte di rimborso Squads v4.");
            m.insert(
                "squads_refund_approved",
                "✅ Proposta di rimborso Squads v4 #{proposal_index} approvata!",
            );
            m.insert("unauthorized_reject", "⛔ Non autorizzato: solo il gestore del negozio può rifiutare le proposte di rimborso Squads v4.");
            m.insert("squads_refund_rejected", "🚫 Proposta di rimborso Squads v4 #{proposal_index} rifiutata. Fattura ripristinata a 'paid'.");
            m.insert(
                "refund_prompt",
                "♻️ Inserisci l'ID della fattura da rimborsare (es.: INV-101):",
            );
            m.insert("squads_refund_initiated", "🏛️ *Proposta Multisig Squads v4 Avviata*\n───────────────────────────\n• Fattura: `{invoice_id}`\n• Importo: *{amount_usdc} USDC*\n• Indice proposta: `#{proposal_index}` (In attesa On-Chain)\n\nApprovare la proposta di rimborso Squads v4?");
            all.insert("it", m);
        }

        // ── pl ──────────────────────────────────────────────────────────
        {
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
            m.insert("btn_quick_uah", "☕ Szybki paragon (200 UAH)");
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
            m.insert("unauthorized_approve", "⛔ Nieautoryzowano: tylko menedżer sklepu może zatwierdzać wnioski o zwrot Squads v4.");
            m.insert(
                "squads_refund_approved",
                "✅ Wniosek o zwrot Squads v4 #{proposal_index} zatwierdzony!",
            );
            m.insert("unauthorized_reject", "⛔ Nieautoryzowano: tylko menedżer sklepu może odrzucać wnioski o zwrot Squads v4.");
            m.insert("squads_refund_rejected", "🚫 Wniosek o zwrot Squads v4 #{proposal_index} odrzucony. Paragon przywrócony do 'paid'.");
            m.insert(
                "refund_prompt",
                "♻️ Podaj ID paragonu do zwrotu (np. INV-101):",
            );
            m.insert("squads_refund_initiated", "🏛️ *Zainicjowano wniosek multisig Squads v4*\n───────────────────────────\n• Paragon: `{invoice_id}`\n• Kwota: *{amount_usdc} USDC*\n• Indeks wniosku: `#{proposal_index}` (Oczekuje On-Chain)\n\nZatwierdzić wniosek o zwrot Squads v4?");
            all.insert("pl", m);
        }

        // ── tr ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ Ödeme Onaylandı!\nFatura #{invoice_id}\nTutar: {amount} {currency}\nİşlem: {tx_sig}");
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
            m.insert("welcome", "☕ *ZeroClaw Solana POS Terminaline Hoş Geldiniz!*\n\nBir işlem seçin veya tutar girin:");
            m.insert("custom_help", "✍️ *Mesajınızda tutarı ve para birimini girin:*\n\nÖrnekler:\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
            m.insert("price_needed", "✍️ Lütfen '{items}' için toplam fiyatı ve para birimini belirtin\n\nÖrnek:\n• `{items} 500 UAH`");
            m.insert("select_lang", "🌐 *Arayüz dilini seçin:*");
            m.insert("btn_custom", "✍️ Özel tutar girin");
            m.insert("btn_quick_uah", "☕ Hızlı fiş (200 UAH)");
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
            m.insert("unauthorized_approve", "⛔ Yetkisiz: Squads v4 iade tekliflerini yalnızca mağaza yöneticisi onaylayabilir.");
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

        // ── ja ──────────────────────────────────────────────────────────
        {
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

        // ── zh ──────────────────────────────────────────────────────────
        {
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
            m.insert("welcome", "☕ *欢迎使用 ZeroClaw Solana POS 终端！*\n\n请在下方键盘选择操作或输入自定义金额：");
            m.insert("custom_help", "✍️ *请在消息中输入金额和货币：*\n\n示例：\n• `150 UAH`\n• `35.50 BRL`\n• `12.50 USD`\n• `2x Cappuccino + Croissant 240 UAH`");
            m.insert(
                "price_needed",
                "✍️ 请指定 '{items}' 的总价和货币\n\n示例：\n• `{items} 500 UAH`",
            );
            m.insert("select_lang", "🌐 *请选择界面语言：*");
            m.insert("btn_custom", "✍️ 输入自定义金额");
            m.insert("btn_quick_uah", "☕ 快速收据 (200 UAH)");
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

        // ── ar ──────────────────────────────────────────────────────────
        {
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
            m.insert("btn_quick_uah", "☕ إيصال سريع (200 UAH)");
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
            m.insert("squads_refund_rejected", "🚫 تم رفض اقتراح استرداد Squads v4 #{proposal_index}. تمت استعادة الإيصال إلى 'paid'.");
            m.insert(
                "refund_prompt",
                "♻️ يرجى إدخال معرف الإيصال لاسترداد المبلغ (مثال: INV-101):",
            );
            m.insert("squads_refund_initiated", "🏛️ *تم بدء اقتراح تعدد التوقيعات Squads v4*\n───────────────────────────\n• الإيصال: `{invoice_id}`\n• المبلغ: *{amount_usdc} USDC*\n• فهرس الاقتراح: `#{proposal_index}` (قيد الانتظار على السلسلة)\n\nهل تريد الموافقة على اقتراح استرداد Squads v4؟");
            all.insert("ar", m);
        }

        // ── hi ──────────────────────────────────────────────────────────
        {
            let mut m = HashMap::new();
            m.insert("payment_success", "✅ भुगतान की पुष्टि की गई!\nबीजक #{invoice_id}\nराशि: {amount} {currency}\nलेन-देन: {tx_sig}");
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

        all
    });
