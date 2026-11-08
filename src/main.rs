use rust_poa_chain::{Blockchain, Transaction, ValidatorSet, WorldState};

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut state = WorldState::new();
    state.set_balance("alice", 100);

    let validators = match ValidatorSet::new(vec!["v0".to_string(), "v1".to_string()]) {
        Some(v) => v,
        None => {
            println!("need at least one validator");
            return;
        }
    };

    // 1000 ms slots, and nobody may propose in the first 200 ms of a slot
    let mut chain = match Blockchain::new(state, validators, 1000, 200) {
        Some(c) => c,
        None => {
            println!(
                "slot duration must be more than zero, and the wait must be shorter than the slot"
            );
            return;
        }
    };

    // (who signs, timestamp, transactions)
    let attempts = vec![
        ("v1", 1000, vec![Transaction::new("alice", "bob", 30, 0)]), // too early in slot
        ("v1", 1250, vec![Transaction::new("alice", "bob", 30, 0)]), // ok
        ("v1", 1500, vec![Transaction::new("alice", "bob", 10, 1)]), // slot already used
        ("v1", 2250, vec![Transaction::new("alice", "bob", 10, 1)]), // slot 2 belongs to v0
        ("v0", 2250, vec![Transaction::new("alice", "bob", 20, 1)]), // ok
        ("v1", 3250, vec![Transaction::new("alice", "bob", 9999, 2)]), // too much money
    ];

    for (validator, timestamp, txs) in attempts {
        match chain.add_block(validator, timestamp, txs) {
            Ok(()) => println!("{} @ {}: block added", validator, timestamp),
            Err(e) => println!("{} @ {}: rejected - {}", validator, timestamp, e),
        }
    }

    for block in chain.blocks() {
        println!("{}", block);
    }
    println!("total transactions: {}", chain.transaction_count());
    println!(
        "alice={} bob={}",
        chain.balance("alice"),
        chain.balance("bob")
    );
    println!("chain valid: {}", chain.is_valid());
}
