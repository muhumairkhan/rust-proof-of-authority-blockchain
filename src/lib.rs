pub fn name() -> &'static str {
    "poa-chain"
}

pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

pub enum WithdrawResult {
    Success,
    ZeroAmount,
    InsufficientBalance,
}

impl Account {
    pub fn new(balance: u64) -> Account {
        Account { balance, nonce: 0 }
    }

    pub fn withdraw(&mut self, amount: u64) -> WithdrawResult {
        if amount == 0 {
            return WithdrawResult::ZeroAmount;
        }
        if self.balance < amount {
            return WithdrawResult::InsufficientBalance;
        }
        self.balance -= amount;
        self.nonce += 1;
        WithdrawResult::Success
    }

    /// Adds money. Returns None if the balance would overflow u64.
    pub fn deposit(&mut self, amount: u64) -> Option<u64> {
        let new_balance = self.balance.checked_add(amount)?;
        self.balance = new_balance;
        Some(self.balance)
    }
}

/// Anything that can describe itself in one line.
pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for Account {
    fn describe(&self) -> String {
        format!("balance={} nonce={}", self.balance, self.nonce)
    }
}

/// Generic function: works for any type that implements Describe.
pub fn print_description<T: Describe>(item: &T) {
    println!("{}", item.describe());
}

/// Transfers between two accounts. Returns the withdraw result.
/// If the withdraw fails, nothing changes.
pub fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> WithdrawResult {
    let result = from.withdraw(amount);
    if let WithdrawResult::Success = result {
        // ignoring overflow here for simplicity
        to.deposit(amount);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(name(), "poa-chain");
    }

    #[test]
    fn withdraw_reduces_balance_and_bumps_nonce() {
        let mut acc = Account::new(100);
        let result = acc.withdraw(30);
        assert!(matches!(result, WithdrawResult::Success));
        assert_eq!(acc.balance, 70);
        assert_eq!(acc.nonce, 1);
    }

    #[test]
    fn deposit_adds_balance() {
        let mut acc = Account::new(10);
        assert_eq!(acc.deposit(5), Some(15));
        assert_eq!(acc.balance, 15);
    }

    #[test]
    fn deposit_overflow_returns_none() {
        let mut acc = Account::new(u64::MAX);
        assert_eq!(acc.deposit(1), None);
        assert_eq!(acc.balance, u64::MAX); // unchanged
    }

    #[test]
    fn describe_formats_account() {
        let acc = Account::new(42);
        assert_eq!(acc.describe(), "balance=42 nonce=0");
    }

    #[test]
    fn transfer_moves_money() {
        let mut alice = Account::new(100);
        let mut bob = Account::new(0);
        let result = transfer(&mut alice, &mut bob, 40);
        assert!(matches!(result, WithdrawResult::Success));
        assert_eq!(alice.balance, 60);
        assert_eq!(bob.balance, 40);
    }

    #[test]
    fn failed_transfer_changes_nothing() {
        let mut alice = Account::new(10);
        let mut bob = Account::new(0);
        let result = transfer(&mut alice, &mut bob, 50);
        assert!(matches!(result, WithdrawResult::InsufficientBalance));
        assert_eq!(alice.balance, 10);
        assert_eq!(bob.balance, 0);
    }
}