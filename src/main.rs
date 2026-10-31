use rust_poa_chain::{Transaction, WorldState};

fn tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
    Transaction {
        from: from.to_string(),
        to: to.to_string(),
        amount,
        nonce,
    }
}

fn print_balances(state: &WorldState) {
    println!(
        "alice={} bob={}",
        state.balance("alice"),
        state.balance("bob")
    );
}

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut state = WorldState::new();
    state.set_balance("alice", 100);

    let batch1 = vec![tx("alice", "bob", 30, 0), tx("alice", "bob", 20, 1)];
    match state.apply_all(&batch1) {
        Ok(()) => println!("Batch 1 applied"),
        Err(e) => println!("Batch 1 failed: {:?}", e),
    }
    print_balances(&state);

    let batch2 = vec![tx("alice", "bob", 10, 2), tx("alice", "bob", 1000, 3)];
    match state.apply_all(&batch2) {
        Ok(()) => println!("Batch 2 applied"),
        Err(e) => println!("Batch 2 failed: {:?}", e),
    }
    print_balances(&state);
}
