pub mod constants;
pub mod solana_pay;
pub mod squads;
pub mod token2022;

// Re-export public API
pub use constants::*;
pub use solana_pay::{
    build_solana_pay_url, generate_atomic_refund_instructions, generate_phantom_universal_link,
    generate_secure_reference_key, get_active_rpc_url, validate_squads_multisig_account,
};
pub use squads::{base64_encode, build_squads_v4_instruction_data, build_squads_v4_proposal, hex_encode, ANCHOR_DISCRIMINATOR};
pub use token2022::{
    atomic_to_f64, calculate_token2022_fee, is_payment_amount_valid, safe_f64_to_u64_atomic,
    usdc_to_atomic_units,
};
