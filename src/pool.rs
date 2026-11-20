use std::sync::mpsc;
use std::thread;

use crate::transaction::Transaction;

/// Spawns one thread per batch. Each thread sends its transactions through
/// a channel, and this function collects everything that arrives.
/// Order between threads is not fixed, but each thread's own
/// transactions arrive in the order it sent them.
pub fn collect_from_threads(batches: Vec<Vec<Transaction>>) -> Vec<Transaction> {
    let (sender, receiver) = mpsc::channel();
    let mut handles = Vec::new();

    for batch in batches {
        let sender = sender.clone(); // each thread gets its own sender
        handles.push(thread::spawn(move || {
            for tx in batch {
                if sender.send(tx).is_err() {
                    break; // receiver is gone, nothing left to do
                }
            }
        }));
    }

    // Drop our own sender. The receiver's iterator ends only when
    // ALL senders are dropped, so keeping this one would hang forever.
    drop(sender);

    let received: Vec<Transaction> = receiver.iter().collect();

    for handle in handles {
        handle.join().expect("sender thread panicked");
    }
    received
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldState;

    fn batch_from(sender: &str) -> Vec<Transaction> {
        vec![
            Transaction::new(sender, "dest", 10, 0),
            Transaction::new(sender, "dest", 10, 1),
        ]
    }

    #[test]
    fn receives_everything_from_every_thread() {
        let batches = vec![batch_from("a"), batch_from("b"), batch_from("c")];
        let received = collect_from_threads(batches);
        assert_eq!(received.len(), 6);
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        assert!(collect_from_threads(Vec::new()).is_empty());
    }

    #[test]
    fn each_threads_transactions_keep_their_order() {
        let received = collect_from_threads(vec![batch_from("a"), batch_from("b")]);
        for who in ["a", "b"] {
            let nonces: Vec<u64> = received
                .iter()
                .filter(|tx| tx.from == who)
                .map(|tx| tx.nonce)
                .collect();
            assert_eq!(nonces, vec![0, 1]);
        }
    }

    #[test]
    fn collected_transactions_apply_to_state() {
        let mut state = WorldState::new();
        for who in ["a", "b", "c"] {
            state.set_balance(who, 100);
        }

        let batches = vec![batch_from("a"), batch_from("b"), batch_from("c")];
        let received = collect_from_threads(batches);

        assert_eq!(state.apply_all(&received), Ok(()));
        assert_eq!(state.balance("dest"), 60);
        assert_eq!(state.total_supply(), 300);
    }
}
