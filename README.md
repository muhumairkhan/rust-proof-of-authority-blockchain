# rust-poa-chain

A Proof-of-Authority blockchain in Rust, built step by step as a learning project.

## Done so far

- Accounts, transactions and atomic world-state updates
- SHA-256 hash-linked blocks and chain validation
- Validator rotation by time slot, with a slot wait period
- Typed errors with readable messages

## Next

- Signatures and addresses (k256)
- Persistence, CLI, API, P2P

## Run

    cargo run
    cargo test
