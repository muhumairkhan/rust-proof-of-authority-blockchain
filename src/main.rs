use rust_poa_chain::{print_description, transfer, Account, WithdrawResult};

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut alice = Account::new(100);
    let mut bob = Account::new(0);

    print_description(&alice);
    print_description(&bob);

    match transfer(&mut alice, &mut bob, 30) {
        WithdrawResult::Success => println!("Transferred 30"),
        WithdrawResult::ZeroAmount => println!("Amount must be more than zero"),
        WithdrawResult::InsufficientBalance => println!("Not enough balance"),
    }

    print_description(&alice);
    print_description(&bob);

    // Option in action
    match bob.deposit(u64::MAX) {
        Some(new_balance) => println!("Deposited, balance is {}", new_balance),
        None => println!("Deposit would overflow"),
    }
}