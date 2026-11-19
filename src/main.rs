use std::path::PathBuf;
use std::process;

use rust_poa_chain::export::{append_summary, count_report_lines, file_size, write_report};
use rust_poa_chain::{Blockchain, Transaction, ValidatorSet, WorldState};

fn main() {
    println!("{} node starting...", rust_poa_chain::name());

    let mut state = WorldState::new();
    state.set_balance("alice", 100);

    let validators = match ValidatorSet::new(vec!["v0".to_string(), "v1".to_string()]) {
        Some(v) => v,
        None => {
            eprintln!("need at least one validator");
            process::exit(1);
        }
    };

    // 1000 ms slots, and nobody may propose in the first 200 ms of a slot
    let mut chain = match Blockchain::new(state, validators, 1000, 200) {
        Some(c) => c,
        None => {
            eprintln!(
                "slot duration must be more than zero, and the wait must be shorter than the slot"
            );
            process::exit(1);
        }
    };

    // (who signs, timestamp, transactions)
    let attempts = vec![
        ("v1", 1250, vec![Transaction::new("alice", "bob", 30, 0)]),
        ("v0", 2250, vec![Transaction::new("alice", "bob", 20, 1)]),
        ("v1", 3250, vec![Transaction::new("alice", "bob", 9999, 2)]), // rejected
    ];

    for (validator, timestamp, txs) in attempts {
        match chain.add_block(validator, timestamp, txs) {
            Ok(()) => println!("{} @ {}: block added", validator, timestamp),
            Err(e) => println!("{} @ {}: rejected - {}", validator, timestamp, e),
        }
    }

    let path: PathBuf = ["data", "report.txt"].iter().collect();

    if let Err(e) = write_report(&chain, &path) {
        eprintln!("could not write {}: {}", path.display(), e);
        process::exit(1);
    }
    if let Err(e) = append_summary(&chain, &path) {
        eprintln!("could not append to {}: {}", path.display(), e);
        process::exit(1);
    }

    match (count_report_lines(&path), file_size(&path)) {
        (Ok(lines), Ok(bytes)) => {
            println!(
                "wrote {} lines ({} bytes) to {}",
                lines,
                bytes,
                path.display()
            )
        }
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("could not read {}: {}", path.display(), e);
            process::exit(1);
        }
    }
}
