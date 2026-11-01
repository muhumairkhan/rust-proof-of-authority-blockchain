use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub fn name() -> &'static str {
    "poa-chain"
}

#[derive(Debug, Clone)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Debug, PartialEq)]
pub enum TxError {
    ZeroAmount,
    InsufficientBalance,
    Overflow,
    SameAccount,
    BadNonce { expected: u64, got: u64 },
}

impl Account {
    pub fn new(balance: u64) -> Account {
        Account { balance, nonce: 0 }
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<(), TxError> {
        if amount == 0 {
            return Err(TxError::ZeroAmount);
        }
        if self.balance < amount {
            return Err(TxError::InsufficientBalance);
        }
        self.balance -= amount;
        self.nonce += 1;
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<(), TxError> {
        if amount == 0 {
            return Err(TxError::ZeroAmount);
        }
        match self.balance.checked_add(amount) {
            Some(new_balance) => {
                self.balance = new_balance;
                Ok(())
            }
            None => Err(TxError::Overflow),
        }
    }
}

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for Account {
    fn describe(&self) -> String {
        format!("balance={} nonce={}", self.balance, self.nonce)
    }
}

pub fn print_description<T: Describe>(item: &T) {
    println!("{}", item.describe());
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    accounts: HashMap<String, Account>,
}

impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            accounts: HashMap::new(),
        }
    }

    /// Gives an address a starting balance (like genesis allocations).
    pub fn set_balance(&mut self, address: &str, balance: u64) {
        self.accounts
            .insert(address.to_string(), Account::new(balance));
    }

    /// Unknown addresses simply have a balance of 0.
    pub fn balance(&self, address: &str) -> u64 {
        match self.accounts.get(address) {
            Some(account) => account.balance,
            None => 0,
        }
    }

    pub fn nonce(&self, address: &str) -> u64 {
        match self.accounts.get(address) {
            Some(account) => account.nonce,
            None => 0,
        }
    }

    /// Applies one transaction. Checks everything first, then changes state.
    pub fn apply(&mut self, tx: &Transaction) -> Result<(), TxError> {
        if tx.amount == 0 {
            return Err(TxError::ZeroAmount);
        }
        if tx.from == tx.to {
            return Err(TxError::SameAccount);
        }
        let expected = self.nonce(&tx.from);
        if tx.nonce != expected {
            return Err(TxError::BadNonce {
                expected,
                got: tx.nonce,
            });
        }
        if self.balance(&tx.from) < tx.amount {
            return Err(TxError::InsufficientBalance);
        }
        if self.balance(&tx.to).checked_add(tx.amount).is_none() {
            return Err(TxError::Overflow);
        }

        // Everything is valid, so now it is safe to change things.
        let sender = self
            .accounts
            .entry(tx.from.clone())
            .or_insert(Account::new(0));
        sender.withdraw(tx.amount)?;

        let receiver = self
            .accounts
            .entry(tx.to.clone())
            .or_insert(Account::new(0));
        receiver.deposit(tx.amount)?;

        Ok(())
    }

    /// All-or-nothing: if any transaction fails, the state is left untouched.
    pub fn apply_all(&mut self, txs: &[Transaction]) -> Result<(), TxError> {
        let mut working = self.clone();
        for tx in txs {
            working.apply(tx)?;
        }
        *self = working;
        Ok(())
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(
        index: u64,
        timestamp: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
    ) -> Block {
        // borrow first to compute the hash, then move the values into the struct
        let hash = Block::compute_hash(index, timestamp, &transactions, &previous_hash);
        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
        }
    }

    /// Turns the block's contents into text, then hashes the text with SHA-256.
    pub fn compute_hash(
        index: u64,
        timestamp: u64,
        transactions: &[Transaction],
        previous_hash: &str,
    ) -> String {
        let mut text = format!("{}|{}|{}", index, timestamp, previous_hash);
        for tx in transactions {
            text.push_str(&format!(
                "|{}>{}:{}:{}",
                tx.from, tx.to, tx.amount, tx.nonce
            ));
        }
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Does the stored hash still match the block's contents?
    pub fn is_hash_valid(&self) -> bool {
        self.hash
            == Block::compute_hash(
                self.index,
                self.timestamp,
                &self.transactions,
                &self.previous_hash,
            )
    }
}

pub struct Blockchain {
    chain: Vec<Block>,
    state: WorldState,
}

impl Blockchain {
    /// Starts a chain containing only the genesis block.
    pub fn new(state: WorldState) -> Blockchain {
        let genesis = Block::new(0, 0, Vec::new(), "0".repeat(64));
        Blockchain {
            chain: vec![genesis],
            state,
        }
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

    /// Applies the transactions, and only if ALL are valid, appends a new block.
    pub fn add_block(&mut self, timestamp: u64, txs: Vec<Transaction>) -> Result<(), TxError> {
        self.state.apply_all(&txs)?;

        let latest = self.latest_block();
        let block = Block::new(latest.index + 1, timestamp, txs, latest.hash.clone());
        self.chain.push(block);
        Ok(())
    }

    /// Checks every block's hash and every link to the block before it.
    pub fn is_valid(&self) -> bool {
        for i in 0..self.chain.len() {
            let block = &self.chain[i];
            if !block.is_hash_valid() {
                return false;
            }
            if i > 0 {
                let previous = &self.chain[i - 1];
                if block.previous_hash != previous.hash || block.index != previous.index + 1 {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            nonce,
        }
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(name(), "poa-chain");
    }

    #[test]
    fn withdraw_and_deposit_work() {
        let mut acc = Account::new(100);
        assert_eq!(acc.withdraw(30), Ok(()));
        assert_eq!(acc.deposit(5), Ok(()));
        assert_eq!(acc.balance, 75);
        assert_eq!(acc.nonce, 1);
    }

    #[test]
    fn describe_formats_account() {
        assert_eq!(Account::new(42).describe(), "balance=42 nonce=0");
    }

    #[test]
    fn unknown_address_has_zero_balance() {
        let state = WorldState::new();
        assert_eq!(state.balance("nobody"), 0);
        assert_eq!(state.nonce("nobody"), 0);
    }

    #[test]
    fn apply_moves_money_and_bumps_nonce() {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        assert_eq!(state.apply(&tx("alice", "bob", 40, 0)), Ok(()));
        assert_eq!(state.balance("alice"), 60);
        assert_eq!(state.balance("bob"), 40);
        assert_eq!(state.nonce("alice"), 1);
    }

    #[test]
    fn wrong_nonce_is_rejected() {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        assert_eq!(
            state.apply(&tx("alice", "bob", 10, 5)),
            Err(TxError::BadNonce {
                expected: 0,
                got: 5
            })
        );
        assert_eq!(state.balance("alice"), 100);
    }

    #[test]
    fn insufficient_balance_is_rejected() {
        let mut state = WorldState::new();
        state.set_balance("alice", 10);
        assert_eq!(
            state.apply(&tx("alice", "bob", 50, 0)),
            Err(TxError::InsufficientBalance)
        );
    }

    #[test]
    fn sending_to_self_is_rejected() {
        let mut state = WorldState::new();
        state.set_balance("alice", 10);
        assert_eq!(
            state.apply(&tx("alice", "alice", 5, 0)),
            Err(TxError::SameAccount)
        );
    }

    #[test]
    fn apply_all_applies_in_order() {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        let txs = vec![tx("alice", "bob", 30, 0), tx("alice", "bob", 20, 1)];
        assert_eq!(state.apply_all(&txs), Ok(()));
        assert_eq!(state.balance("alice"), 50);
        assert_eq!(state.balance("bob"), 50);
    }

    #[test]
    fn apply_all_is_all_or_nothing() {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        let txs = vec![tx("alice", "bob", 30, 0), tx("alice", "bob", 1000, 1)];
        assert_eq!(state.apply_all(&txs), Err(TxError::InsufficientBalance));
        // the first tx was valid, but it must NOT have stuck
        assert_eq!(state.balance("alice"), 100);
        assert_eq!(state.balance("bob"), 0);
        assert_eq!(state.nonce("alice"), 0);
    }

    #[test]
    fn block_hash_is_deterministic() {
        let a = Block::new(1, 100, vec![tx("alice", "bob", 5, 0)], "abc".to_string());
        let b = Block::new(1, 100, vec![tx("alice", "bob", 5, 0)], "abc".to_string());
        assert_eq!(a.hash, b.hash);
        assert_eq!(a.hash.len(), 64);
    }

    #[test]
    fn block_hash_changes_with_contents() {
        let a = Block::new(1, 100, vec![tx("alice", "bob", 5, 0)], "abc".to_string());
        let b = Block::new(1, 100, vec![tx("alice", "bob", 6, 0)], "abc".to_string());
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn new_chain_has_only_genesis() {
        let chain = Blockchain::new(WorldState::new());
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.latest_block().index, 0);
        assert!(chain.is_valid());
    }

    #[test]
    fn add_block_links_and_updates_state() {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        let mut chain = Blockchain::new(state);

        assert_eq!(
            chain.add_block(1000, vec![tx("alice", "bob", 40, 0)]),
            Ok(())
        );
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.balance("bob"), 40);
        assert_eq!(chain.blocks()[1].previous_hash, chain.blocks()[0].hash);
        assert!(chain.is_valid());
    }

    #[test]
    fn failed_block_is_not_added() {
        let mut state = WorldState::new();
        state.set_balance("alice", 10);
        let mut chain = Blockchain::new(state);

        let result = chain.add_block(1000, vec![tx("alice", "bob", 50, 0)]);
        assert_eq!(result, Err(TxError::InsufficientBalance));
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.balance("alice"), 10);
    }

    #[test]
    fn tampering_is_detected() {
        let mut state = WorldState::new();
        state.set_balance("alice", 100);
        let mut chain = Blockchain::new(state);
        chain
            .add_block(1000, vec![tx("alice", "bob", 10, 0)])
            .unwrap();
        chain
            .add_block(2000, vec![tx("alice", "bob", 10, 1)])
            .unwrap();
        assert!(chain.is_valid());

        // sneakily change an old transaction
        chain.chain[1].transactions[0].amount = 999;
        assert!(!chain.is_valid());
    }
}