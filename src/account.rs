use crate::error::TxError;

#[derive(Debug, Clone)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn withdraw_and_deposit_work() {
        let mut acc = Account::new(100);
        assert_eq!(acc.withdraw(30), Ok(()));
        assert_eq!(acc.deposit(5), Ok(()));
        assert_eq!(acc.balance, 75);
        assert_eq!(acc.nonce, 1);
    }
}
