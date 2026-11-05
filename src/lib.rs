pub mod account;
pub mod block;
pub mod blockchain;
pub mod crypto;
pub mod describe;
pub mod error;
pub mod state;
pub mod transaction;
pub mod validator;

pub use account::Account;
pub use block::Block;
pub use blockchain::Blockchain;
pub use describe::{Describe, print_description};
pub use error::{BlockError, TxError};
pub use state::WorldState;
pub use transaction::Transaction;
pub use validator::ValidatorSet;

pub fn name() -> &'static str {
    "poa-chain"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(name(), "poa-chain");
    }
}
