/// USDC decimal places
pub const USDC_DECIMALS: u8 = 6;

/// SOL decimal places
pub const SOL_DECIMALS: u8 = 9;

/// Unsigned 64-bit integer upper bound
pub const MAX_U64: u64 = u64::MAX;

/// USDC Mint (Mainnet)
pub const USDC_MINT_MAINNET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

/// USDC Mint (Devnet)
pub const USDC_MINT_DEVNET: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

/// Wrapped SOL Mint
pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

/// EURC Mint
pub const EURC_MINT: &str = "HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr";

/// Base58 alphabet for Solana public key validation
pub const BASE58_ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Default slippage tolerance percentage
pub const DEFAULT_SLIPPAGE_TOLERANCE_PCT: f64 = 1.0;

/// Default commitment threshold for finalized vs confirmed
pub const DEFAULT_COMMITMENT_THRESHOLD_USDC: f64 = 50.0;

/// Nonce account expiry TTL in minutes
pub const NONCE_TTL_MINUTES: i64 = 15;

/// Default socket timeout in seconds
pub const DEFAULT_SOCKET_TIMEOUT: f64 = 10.0;
