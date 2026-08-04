use proptest::prelude::*;

use pos_backend::domain::order_parser::parse_pos_order_input;

proptest! {
    #[test]
    fn test_amount_never_negative(
        amount in 1.0f64..999_999.99,
        currency in "[A-Z]{3}"
    ) {
        let input = format!("{} {}", amount, currency);
        let result = parse_pos_order_input(&input, "Order", None);
        if result.has_price {
            prop_assert!(result.amount.unwrap() > 0.0);
        }
    }

    #[test]
    fn test_valid_currency_always_parsed(
        amount in 1.0f64..999_999.99,
        currency in "UAH|USD|EUR|BRL|GBP|PLN"
    ) {
        let input = format!("{} {}", amount, currency);
        let result = parse_pos_order_input(&input, "Order", None);
        prop_assert!(result.has_price);
        prop_assert!(result.currency.is_some());
    }

    #[test]
    fn test_bare_number_defaults_to_uah(
        amount in 1.0f64..999_999.99
    ) {
        let input = format!("{}", amount);
        let result = parse_pos_order_input(&input, "Order", None);
        prop_assert!(result.has_price);
        prop_assert_eq!(result.currency.as_deref(), Some("UAH"));
    }

    #[test]
    fn test_overflow_rejected(
        amount in 1_000_000.0f64..f64::MAX
    ) {
        let input = format!("{} USD", amount);
        let result = parse_pos_order_input(&input, "Order", None);
        prop_assert!(!result.has_price);
    }

    #[test]
    fn test_empty_input_no_price(
        input in "\\s*"
    ) {
        let result = parse_pos_order_input(&input, "Order", None);
        prop_assert!(!result.has_price);
    }
}
