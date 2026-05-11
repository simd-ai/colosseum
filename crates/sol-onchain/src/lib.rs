//! On-chain helpers for SolGrid: program IDs, PDA derivation, instruction builders.
//!
//! This crate intentionally talks to the chain through `solana-sdk` rather than
//! `anchor-client`, because anchor-client pulls in the full Anchor IDL machinery
//! and that conflicts with our off-chain solana 2.x dep tree. Anchor instructions
//! are just discriminator + borsh args, so we serialize them by hand.

pub mod ids;
pub mod pda;
pub mod ix;

pub use ids::*;
pub use pda::*;
pub use ix::*;
