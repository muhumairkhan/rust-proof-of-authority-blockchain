use crate::block::Block;
use crate::error::BlockError;
use crate::state::WorldState;
use crate::transaction::Transaction;
use crate::validator::ValidatorSet;

pub struct Blockchain {
    chain: Vec<Block>,
    state: WorldState,
    validators: ValidatorSet,
    slot_duration_ms: u64,
}

impl Blockchain {
    /// Starts a chain containing only the genesis block.
    /// Returns None if the slot duration is 0 (we would divide by zero).
    pub fn new(
        state: WorldState,
        validators: ValidatorSet,
        slot_duration_ms: u64,
    ) -> Option<Blockchain> {
        if slot_duration_ms == 0 {
            return None;
        }
        let genesis = Block::new(0, 0, "genesis".to_string(), Vec::new(), "0".repeat(64));
        Some(Blockchain {
            chain: vec![genesis],
            state,
            validators,
            slot_duration_ms,
        })
    }

    pub fn latest_block(&self) -> &Block {
        // the chain always has at least the genesis block, so this cannot fail
        self.chain.last().expect("chain always has a genesis block")
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    pub fn blocks(&self) -> &[Block] {
        &self.chain
    }

    pub fn balance(&self, address: &str) -> u64 {
        self.state.balance(address)
    }

    /// Which slot a timestamp falls into.
    pub fn slot(&self, timestamp: u64) -> u64 {
        timestamp / self.slot_duration_ms
    }

    /// Checks the slot rules for a block made at `timestamp` by `validator`,
    /// coming after `previous`. Does not look at transactions.
    fn check_slot(
        &self,
        previous: &Block,
        timestamp: u64,
        validator: &str,
    ) -> Result<(), BlockError> {
        let slot = self.slot(timestamp);
        if slot <= self.slot(previous.timestamp) {
            return Err(BlockError::SlotAlreadyUsed { slot });
        }
        let expected = self.validators.validator_for_slot(slot);
        if validator != expected {
            return Err(BlockError::WrongValidator {
                expected: expected.to_string(),
                got: validator.to_string(),
            });
        }
        Ok(())
    }

    /// Checks the slot rules, applies the transactions, and only if ALL is fine,
    /// appends a new block.
    pub fn add_block(
        &mut self,
        validator: &str,
        timestamp: u64,
        txs: Vec<Transaction>,
    ) -> Result<(), BlockError> {
        self.check_slot(self.latest_block(), timestamp, validator)?;

        // `?` converts TxError into BlockError using our From impl
        self.state.apply_all(&txs)?;

        let latest = self.latest_block();
        let block = Block::new(
            latest.index + 1,
            timestamp,
            validator.to_string(),
            txs,
            latest.hash.clone(),
        );
        self.chain.push(block);
        Ok(())
    }

    /// Checks every block's hash, every link, and every slot rule.
    pub fn is_valid(&self) -> bool {
        // every block's own hash must match its contents
        if !self.chain.iter().all(|block| block.is_hash_valid()) {
            return false;
        }
        // windows(2) walks the chain as overlapping pairs: (0,1), (1,2), (2,3), ...
        self.chain.windows(2).all(|pair| {
            let previous = &pair[0];
            let block = &pair[1];
            block.previous_hash == previous.hash
                && block.index == previous.index + 1
                && self
                    .check_slot(previous, block.timestamp, &block.validator)
                    .is_ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
        Transaction::new(from, to, amount, nonce)
    }

    /// Two validators, 1000 ms slots, alice has 100.
    /// Slot 1 (ts 1000..1999) -> v1, slot 2 (ts 2000..2999) -> v0, and so on.
    fn make_chain() -> Blockchain {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        let validators = ValidatorSet::new(vec!["v0".to_string(), "v1".to_string()]).unwrap();
        Blockchain::new(state, validators, 1000).unwrap()
    }

    #[test]
    fn new_chain_has_only_genesis() {
        let chain = make_chain();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.latest_block().index, 0);
        assert!(chain.is_valid());
    }

    #[test]
    fn add_block_links_and_updates_state() {
        let mut chain = make_chain();

        assert_eq!(
            chain.add_block("v1", 1000, vec![tx("alice", "bob", 40, 0)]),
            Ok(())
        );
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.balance("bob"), 40);
        assert_eq!(chain.blocks()[1].previous_hash, chain.blocks()[0].hash);
        assert_eq!(chain.blocks()[1].validator, "v1");
        assert!(chain.is_valid());
    }

    #[test]
    fn failed_block_is_not_added() {
        let mut chain = make_chain();

        let result = chain.add_block("v1", 1000, vec![tx("alice", "bob", 500, 0)]);
        assert_eq!(
            result,
            Err(BlockError::Tx(crate::error::TxError::InsufficientBalance))
        );
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.balance("alice"), 100);
    }

    #[test]
    fn tampering_is_detected() {
        let mut chain = make_chain();
        chain
            .add_block("v1", 1000, vec![tx("alice", "bob", 10, 0)])
            .unwrap();
        chain
            .add_block("v0", 2000, vec![tx("alice", "bob", 10, 1)])
            .unwrap();
        assert!(chain.is_valid());

        // sneakily change an old transaction
        chain.chain[1].transactions[0].amount = 999;
        assert!(!chain.is_valid());
    }

    #[test]
    fn zero_slot_duration_is_refused() {
        let validators = ValidatorSet::new(vec!["v0".to_string()]).unwrap();
        assert!(Blockchain::new(WorldState::new(), validators, 0).is_none());
    }

    #[test]
    fn wrong_validator_is_rejected() {
        let mut chain = make_chain();
        // timestamp 1000 is slot 1, which belongs to v1, not v0
        let result = chain.add_block("v0", 1000, vec![tx("alice", "bob", 10, 0)]);
        assert_eq!(
            result,
            Err(BlockError::WrongValidator {
                expected: "v1".to_string(),
                got: "v0".to_string()
            })
        );
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn second_block_in_same_slot_is_rejected() {
        let mut chain = make_chain();
        chain
            .add_block("v1", 1000, vec![tx("alice", "bob", 10, 0)])
            .unwrap();
        // 1500 is still slot 1
        let result = chain.add_block("v1", 1500, vec![tx("alice", "bob", 10, 1)]);
        assert_eq!(result, Err(BlockError::SlotAlreadyUsed { slot: 1 }));
    }

    #[test]
    fn block_in_the_past_is_rejected() {
        let mut chain = make_chain();
        chain
            .add_block("v0", 2000, vec![tx("alice", "bob", 10, 0)])
            .unwrap();
        // slot 1 is earlier than slot 2
        let result = chain.add_block("v1", 1000, vec![tx("alice", "bob", 10, 1)]);
        assert_eq!(result, Err(BlockError::SlotAlreadyUsed { slot: 1 }));
    }

    #[test]
    fn changing_the_validator_of_an_old_block_is_detected() {
        let mut chain = make_chain();
        chain
            .add_block("v1", 1000, vec![tx("alice", "bob", 10, 0)])
            .unwrap();
        assert!(chain.is_valid());

        chain.chain[1].validator = "v0".to_string();
        assert!(!chain.is_valid());
    }
}
