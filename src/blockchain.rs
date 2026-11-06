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
    slot_wait_ms: u64,
}

impl Blockchain {
    /// Starts a chain containing only the genesis block.
    /// Returns None if the slot duration is 0 (we would divide by zero),
    /// or if the wait is not shorter than the slot (no block could ever be valid).
    pub fn new(
        state: WorldState,
        validators: ValidatorSet,
        slot_duration_ms: u64,
        slot_wait_ms: u64,
    ) -> Option<Blockchain> {
        if slot_duration_ms == 0 || slot_wait_ms >= slot_duration_ms {
            return None;
        }
        let genesis = Block::new(0, 0, "genesis".to_string(), Vec::new(), "0".repeat(64));
        Some(Blockchain {
            chain: vec![genesis],
            state,
            validators,
            slot_duration_ms,
            slot_wait_ms,
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

    /// How many milliseconds after the start of its slot a timestamp is.
    pub fn time_into_slot(&self, timestamp: u64) -> u64 {
        timestamp % self.slot_duration_ms
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

        let time_into_slot = self.time_into_slot(timestamp);
        if time_into_slot < self.slot_wait_ms {
            return Err(BlockError::TooEarlyInSlot {
                time_into_slot,
                wait: self.slot_wait_ms,
            });
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
    use crate::error::TxError;

    fn tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
        Transaction::new(from, to, amount, nonce)
    }

    /// Builds a chain with two validators, 1000 ms slots and the given wait.
    /// alice has 100. Slot 1 (ts 1000..1999) -> v1, slot 2 (ts 2000..2999) -> v0, and so on.
    fn make_chain_with_wait(wait: u64) -> Blockchain {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        let validators = ValidatorSet::new(vec!["v0".to_string(), "v1".to_string()]).unwrap();
        Blockchain::new(state, validators, 1000, wait).unwrap()
    }

    /// No wait, so most tests can use round timestamps like 1000 and 2000.
    fn make_chain() -> Blockchain {
        make_chain_with_wait(0)
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
        assert_eq!(result, Err(BlockError::Tx(TxError::InsufficientBalance)));
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
        assert!(Blockchain::new(WorldState::new(), validators, 0, 0).is_none());
    }

    #[test]
    fn wait_must_be_shorter_than_the_slot() {
        let validators = ValidatorSet::new(vec!["v0".to_string()]).unwrap();
        assert!(Blockchain::new(WorldState::new(), validators.clone(), 1000, 1000).is_none());
        assert!(Blockchain::new(WorldState::new(), validators.clone(), 1000, 5000).is_none());
        assert!(Blockchain::new(WorldState::new(), validators, 1000, 999).is_some());
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

    #[test]
    fn time_into_slot_is_the_remainder() {
        let chain = make_chain();
        assert_eq!(chain.time_into_slot(1000), 0);
        assert_eq!(chain.time_into_slot(1250), 250);
        assert_eq!(chain.time_into_slot(1999), 999);
    }

    #[test]
    fn block_inside_the_wait_period_is_rejected() {
        let mut chain = make_chain_with_wait(300);
        // slot 1 belongs to v1, but 1100 is only 100ms into the slot
        let result = chain.add_block("v1", 1100, vec![tx("alice", "bob", 10, 0)]);
        assert_eq!(
            result,
            Err(BlockError::TooEarlyInSlot {
                time_into_slot: 100,
                wait: 300
            })
        );
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.balance("alice"), 100);
    }

    #[test]
    fn block_exactly_at_the_end_of_the_wait_is_accepted() {
        let mut chain = make_chain_with_wait(300);
        assert_eq!(
            chain.add_block("v1", 1300, vec![tx("alice", "bob", 10, 0)]),
            Ok(())
        );
        assert!(chain.is_valid());
    }

    #[test]
    fn wait_rule_is_checked_again_when_validating_the_chain() {
        let mut chain = make_chain_with_wait(300);
        chain
            .add_block("v1", 1400, vec![tx("alice", "bob", 10, 0)])
            .unwrap();
        assert!(chain.is_valid());

        // move the timestamp into the wait period, then fix up the hash so ONLY the wait rule is broken
        let block = &mut chain.chain[1];
        block.timestamp = 1100;
        block.hash = Block::compute_hash(
            block.index,
            block.timestamp,
            &block.validator,
            &block.transactions,
            &block.previous_hash,
        );
        assert!(!chain.is_valid());
    }
}
