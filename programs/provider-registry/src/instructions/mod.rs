pub mod register_provider;
pub mod update_provider;

// Re-export the full instruction modules so the `#[program]` macro can find
// the `__client_accounts_*` and `__cpi_client_accounts_*` items it generates.
pub use register_provider::*;
pub use update_provider::*;
pub use crate::state::ProviderStatus;
