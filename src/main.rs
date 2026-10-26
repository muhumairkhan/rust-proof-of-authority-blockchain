use rust_poa_chain::{Account, WithdrawResult};

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut alice = Account::new(100);

    let result = alice.withdraw(30);
    match result {
        WithdrawResult::Success => println!("Withdrew 30, balance is now {}", alice.balance),
        WithdrawResult::ZeroAmount => println!("Amount must be more than zero"),
        WithdrawResult::InsufficientBalance => println!("Not enough balance"),
    }

    let result = alice.withdraw(500);
    match result {
        WithdrawResult::Success => println!("Withdrew 500"),
        WithdrawResult::ZeroAmount => println!("Amount must be more than zero"),
        WithdrawResult::InsufficientBalance => println!("Not enough balance"),
    }
}
