use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::blockchain::Blockchain;

pub type SharedChain = Arc<Mutex<Blockchain>>;

/// Every `tick`, tries to add an empty block signed by `validator`.
/// Runs until `token` is cancelled, then returns how many blocks it added.
/// `clock` gives the current time in ms, so tests can fake it.
pub async fn run_proposer<F>(
    chain: SharedChain,
    validator: String,
    tick: Duration,
    clock: F,
    token: CancellationToken,
) -> u64
where
    F: Fn() -> u64,
{
    let mut ticker = interval(tick);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut proposed = 0;

    loop {
        tokio::select! {
            biased; // check cancellation first, in order
            _ = token.cancelled() => break,
            _ = ticker.tick() => {
                let now = clock();
                // The guard lives only inside this block, so it is dropped
                // before the next .await.
                let result = {
                    let mut guard = chain.lock().expect("mutex was poisoned");
                    guard.add_block(&validator, now, Vec::new())
                };
                if result.is_ok() {
                    proposed += 1;
                }
            }
        }
    }
    proposed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ValidatorSet, WorldState};

    fn shared_chain() -> SharedChain {
        let validators = ValidatorSet::new(vec!["v0".to_string()]).unwrap();
        let chain = Blockchain::new(WorldState::new(), validators, 1000, 0).unwrap();
        Arc::new(Mutex::new(chain))
    }

    /// A fake clock that moves forward one full slot (1000 ms) on every call.
    fn fake_clock() -> impl Fn() -> u64 {
        let now = Mutex::new(0u64);
        move || {
            let mut n = now.lock().unwrap();
            *n += 1000;
            *n
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
    }

    #[test]
    fn proposer_adds_blocks_until_cancelled() {
        let rt = runtime();
        rt.block_on(async {
            let chain = shared_chain();
            let token = CancellationToken::new();

            let handle = tokio::spawn(run_proposer(
                Arc::clone(&chain),
                "v0".to_string(),
                Duration::from_millis(10),
                fake_clock(),
                token.clone(),
            ));

            tokio::time::sleep(Duration::from_millis(100)).await;
            token.cancel();
            let proposed = handle.await.unwrap();

            let guard = chain.lock().unwrap();
            assert!(proposed >= 2, "only proposed {}", proposed);
            assert_eq!(guard.len() as u64, proposed + 1); // + genesis
            assert!(guard.is_valid());
        });
    }

    #[test]
    fn cancelled_token_stops_before_any_block() {
        let rt = runtime();
        rt.block_on(async {
            let chain = shared_chain();
            let token = CancellationToken::new();
            token.cancel(); // cancelled before the task even starts

            let proposed = run_proposer(
                Arc::clone(&chain),
                "v0".to_string(),
                Duration::from_millis(10),
                fake_clock(),
                token,
            )
            .await;

            assert_eq!(proposed, 0); // `biased;` checks cancellation first
            assert_eq!(chain.lock().unwrap().len(), 1);
        });
    }

    #[test]
    fn wrong_validator_never_adds_blocks() {
        let rt = runtime();
        rt.block_on(async {
            let chain = shared_chain();
            let token = CancellationToken::new();

            let handle = tokio::spawn(run_proposer(
                Arc::clone(&chain),
                "intruder".to_string(),
                Duration::from_millis(10),
                fake_clock(),
                token.clone(),
            ));

            tokio::time::sleep(Duration::from_millis(50)).await;
            token.cancel();

            assert_eq!(handle.await.unwrap(), 0);
            assert_eq!(chain.lock().unwrap().len(), 1);
        });
    }
}
