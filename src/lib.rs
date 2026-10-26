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
    fn cannot_withdraw_more_than_balance() {
        let mut acc = Account::new(10);
        let result = acc.withdraw(50);
        assert!(matches!(result, WithdrawResult::InsufficientBalance));
        assert_eq!(acc.balance, 10); // unchanged
        assert_eq!(acc.nonce, 0); // unchanged
    }

    #[test]
    fn cannot_withdraw_zero() {
        let mut acc = Account::new(10);
        let result = acc.withdraw(0);
        assert!(matches!(result, WithdrawResult::ZeroAmount));
    }
}
