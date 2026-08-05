use std::collections::HashMap;

pub fn register(all: &mut HashMap<&'static str, HashMap<&'static str, &'static str>>) {
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
    m.insert(
        "receipt_fiat_rate",
        "• Фіат: {amount} {currency} (Курс: {rate})",
    );
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
    m.insert("btn_quick_uah", "☕ Швидкий чек ({amount} {currency})");
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
