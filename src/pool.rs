use std::sync::{Arc, Mutex, mpsc};
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

/// Same job, different design: all threads push into ONE shared Vec.
/// `Arc` lets every thread own a handle to it, `Mutex` lets only one
/// thread touch it at a time.
pub fn collect_with_mutex(batches: Vec<Vec<Transaction>>) -> Vec<Transaction> {
    let shared: Arc<Mutex<Vec<Transaction>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for batch in batches {
        let shared = Arc::clone(&shared); // a new handle to the same Vec
        handles.push(thread::spawn(move || {
            for tx in batch {
                // The guard returned by lock() is dropped at the end of this statement,
                // so the lock is held only for one push.
                shared.lock().expect("mutex was poisoned").push(tx);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    // All threads are finished, so lock once more and copy the result out.
    shared.lock().expect("mutex was poisoned").clone()
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

    #[test]
    fn mutex_version_receives_everything() {
        let batches = vec![batch_from("a"), batch_from("b"), batch_from("c")];
        assert_eq!(collect_with_mutex(batches).len(), 6);
    }

    #[test]
    fn mutex_version_handles_empty_input() {
        assert!(collect_with_mutex(Vec::new()).is_empty());
    }

    #[test]
    fn mutex_version_keeps_each_threads_order() {
        let received = collect_with_mutex(vec![batch_from("a"), batch_from("b")]);
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
    fn both_designs_collect_the_same_transactions() {
        let make = || vec![batch_from("a"), batch_from("b"), batch_from("c")];

        // thread order differs between runs, so sort before comparing
        let key = |tx: &Transaction| (tx.from.clone(), tx.nonce);
        let mut via_channel = collect_from_threads(make());
        let mut via_mutex = collect_with_mutex(make());
        via_channel.sort_by_key(key);
        via_mutex.sort_by_key(key);

        let a: Vec<String> = via_channel.iter().map(|tx| tx.hash_text()).collect();
        let b: Vec<String> = via_mutex.iter().map(|tx| tx.hash_text()).collect();
        assert_eq!(a, b);
    }
}
