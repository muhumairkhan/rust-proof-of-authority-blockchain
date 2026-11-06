use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TxError {
    ZeroAmount,
    InsufficientBalance,
    Overflow,
    SameAccount,
    BadNonce { expected: u64, got: u64 },
}

/// Everything that can go wrong when adding a block.
#[derive(Debug, PartialEq)]
pub enum BlockError {
    /// The block's slot is not later than the previous block's slot.
    SlotAlreadyUsed { slot: u64 },
    /// The block was made before the slot's wait period was over.
    TooEarlyInSlot { time_into_slot: u64, wait: u64 },
    /// Someone signed a slot that belongs to another validator.
    WrongValidator { expected: String, got: String },
    /// One of the transactions inside was invalid.
    Tx(TxError),
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TxError::ZeroAmount => write!(f, "amount must be more than zero"),
            TxError::InsufficientBalance => write!(f, "insufficient balance"),
            TxError::Overflow => write!(f, "balance would overflow"),
            TxError::SameAccount => write!(f, "cannot send to yourself"),
            TxError::BadNonce { expected, got } => {
                write!(f, "bad nonce: expected {}, got {}", expected, got)
            }
        }
    }
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockError::SlotAlreadyUsed { slot } => {
                write!(f, "slot {} is already used or in the past", slot)
            }
            BlockError::TooEarlyInSlot {
                time_into_slot,
                wait,
            } => {
                write!(
                    f,
                    "proposed {}ms into the slot, but the slot wait is {}ms",
                    time_into_slot, wait
                )
            }
            BlockError::WrongValidator { expected, got } => {
                write!(f, "slot belongs to {}, but {} signed it", expected, got)
            }
            BlockError::Tx(e) => write!(f, "invalid transaction: {}", e),
        }
    }
}

// Marks these as proper error types, so they work with the rest of the Rust ecosystem.
impl std::error::Error for TxError {}
impl std::error::Error for BlockError {}

/// Lets `?` turn a TxError into a BlockError automatically.
impl From<TxError> for BlockError {
    fn from(e: TxError) -> BlockError {
        BlockError::Tx(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_are_readable() {
        assert_eq!(
            TxError::BadNonce {
                expected: 1,
                got: 5
            }
            .to_string(),
            "bad nonce: expected 1, got 5"
        );
        assert_eq!(
            BlockError::Tx(TxError::ZeroAmount).to_string(),
            "invalid transaction: amount must be more than zero"
        );
        assert_eq!(
            BlockError::TooEarlyInSlot {
                time_into_slot: 100,
                wait: 300
            }
            .to_string(),
            "proposed 100ms into the slot, but the slot wait is 300ms"
        );
    }

    #[test]
    fn tx_error_converts_into_block_error() {
        let e: BlockError = TxError::Overflow.into();
        assert_eq!(e, BlockError::Tx(TxError::Overflow));
    }
}
