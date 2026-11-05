use crate::crypto::sha256_hex;
use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub validator: String,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(
        index: u64,
        timestamp: u64,
        validator: String,
        transactions: Vec<Transaction>,
        previous_hash: String,
    ) -> Block {
        // borrow first to compute the hash, then move the values into the struct
        let hash = Block::compute_hash(index, timestamp, &validator, &transactions, &previous_hash);
        Block {
            index,
            timestamp,
            validator,
            transactions,
            previous_hash,
            hash,
        }
    }

    /// Turns the block's contents into text, then hashes the text with SHA-256.
    pub fn compute_hash(
        index: u64,
        timestamp: u64,
        validator: &str,
        transactions: &[Transaction],
        previous_hash: &str,
    ) -> String {
        let mut text = format!("{}|{}|{}|{}", index, timestamp, validator, previous_hash);
        for tx in transactions {
            text.push('|');
            text.push_str(&tx.hash_text());
        }
        sha256_hex(&text)
    }

    /// Does the stored hash still match the block's contents?
    pub fn is_hash_valid(&self) -> bool {
        self.hash
            == Block::compute_hash(
                self.index,
                self.timestamp,
                &self.validator,
                &self.transactions,
                &self.previous_hash,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
        Transaction::new(from, to, amount, nonce)
    }

    #[test]
    fn block_hash_is_deterministic() {
        let a = Block::new(
            1,
            100,
            "v0".to_string(),
            vec![tx("alice", "bob", 5, 0)],
            "abc".to_string(),
        );
        let b = Block::new(
            1,
            100,
            "v0".to_string(),
            vec![tx("alice", "bob", 5, 0)],
            "abc".to_string(),
        );
        assert_eq!(a.hash, b.hash);
        assert_eq!(a.hash.len(), 64);
    }

    #[test]
    fn block_hash_changes_with_contents() {
        let a = Block::new(
            1,
            100,
            "v0".to_string(),
            vec![tx("alice", "bob", 5, 0)],
            "abc".to_string(),
        );
        let b = Block::new(
            1,
            100,
            "v0".to_string(),
            vec![tx("alice", "bob", 6, 0)],
            "abc".to_string(),
        );
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn validator_is_part_of_the_hash() {
        let a = Block::new(1, 100, "v0".to_string(), Vec::new(), "abc".to_string());
        let b = Block::new(1, 100, "v1".to_string(), Vec::new(), "abc".to_string());
        assert_ne!(a.hash, b.hash);
    }
}
