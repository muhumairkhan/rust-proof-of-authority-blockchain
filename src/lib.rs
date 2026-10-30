pub fn name() -> &'static str {
    "poa-chain"
}

pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Debug, PartialEq)]
pub enum TxError {
    ZeroAmount,
    InsufficientBalance,
    Overflow,
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

/// Moves money between accounts. All-or-nothing: if it fails, neither account changes.
pub fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), TxError> {
    // Check everything that could fail BEFORE changing anything.
    if amount == 0 {
        return Err(TxError::ZeroAmount);
    }
    if to.balance.checked_add(amount).is_none() {
        return Err(TxError::Overflow);
    }

    from.withdraw(amount)?;
    to.deposit(amount)?;
    Ok(())
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
        assert_eq!(acc.withdraw(30), Ok(()));
        assert_eq!(acc.balance, 70);
        assert_eq!(acc.nonce, 1);
    }

    #[test]
    fn cannot_withdraw_more_than_balance() {
        let mut acc = Account::new(10);
        assert_eq!(acc.withdraw(50), Err(TxError::InsufficientBalance));
        assert_eq!(acc.balance, 10);
        assert_eq!(acc.nonce, 0);
    }

    #[test]
    fn deposit_adds_balance() {
        let mut acc = Account::new(10);
        assert_eq!(acc.deposit(5), Ok(()));
        assert_eq!(acc.balance, 15);
    }

    #[test]
    fn deposit_overflow_is_an_error() {
        let mut acc = Account::new(u64::MAX);
        assert_eq!(acc.deposit(1), Err(TxError::Overflow));
        assert_eq!(acc.balance, u64::MAX);
    }

    #[test]
    fn deposit_zero_is_an_error() {
        let mut acc = Account::new(10);
        assert_eq!(acc.deposit(0), Err(TxError::ZeroAmount));
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
        assert_eq!(transfer(&mut alice, &mut bob, 40), Ok(()));
        assert_eq!(alice.balance, 60);
        assert_eq!(bob.balance, 40);
    }

    #[test]
    fn failed_transfer_changes_nothing() {
        let mut alice = Account::new(10);
        let mut bob = Account::new(0);
        assert_eq!(
            transfer(&mut alice, &mut bob, 50),
            Err(TxError::InsufficientBalance)
        );
        assert_eq!(alice.balance, 10);
        assert_eq!(bob.balance, 0);
    }

    #[test]
    fn transfer_that_would_overflow_changes_nothing() {
        let mut alice = Account::new(100);
        let mut bob = Account::new(u64::MAX);
        assert_eq!(transfer(&mut alice, &mut bob, 1), Err(TxError::Overflow));
        assert_eq!(alice.balance, 100); // the old version would have lost this money
        assert_eq!(alice.nonce, 0);
    }
}
