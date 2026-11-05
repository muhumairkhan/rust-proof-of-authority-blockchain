#[derive(Debug, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
}

impl Transaction {
    /// `impl Into<String>` accepts both `&str` and `String`.
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        nonce: u64,
    ) -> Transaction {
        Transaction {
            from: from.into(),
            to: to.into(),
            amount,
            nonce,
        }
    }

    /// The text that goes into the block hash for this transaction.
    pub fn hash_text(&self) -> String {
        format!("{}>{}:{}:{}", self.from, self.to, self.amount, self.nonce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_str_and_string() {
        let a = Transaction::new("alice", "bob", 5, 0);
        let b = Transaction::new(String::from("alice"), String::from("bob"), 5, 0);
        assert_eq!(a.hash_text(), b.hash_text());
        assert_eq!(a.hash_text(), "alice>bob:5:0");
    }
}
