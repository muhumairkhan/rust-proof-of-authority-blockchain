use rust_poa_chain::{Blockchain, Transaction, WorldState};

fn tx(from: &str, to: &str, amount: u64, nonce: u64) -> Transaction {
    Transaction {
        from: from.to_string(),
        to: to.to_string(),
        amount,
        nonce,
    }
}

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut state = WorldState::new();
    state.set_balance("alice", 100);
    let mut chain = Blockchain::new(state);

    match chain.add_block(1000, vec![tx("alice", "bob", 30, 0)]) {
        Ok(()) => println!("Block 1 added"),
        Err(e) => println!("Block 1 rejected: {:?}", e),
    }
    match chain.add_block(2000, vec![tx("alice", "bob", 1000, 1)]) {
        Ok(()) => println!("Block 2 added"),
        Err(e) => println!("Block 2 rejected: {:?}", e),
    }

    for block in chain.blocks() {
        println!(
            "#{} hash={}... prev={}... txs={}",
            block.index,
            &block.hash[..10],
            &block.previous_hash[..10],
            block.transactions.len()
        );
    }
    println!("chain valid: {}", chain.is_valid());
}