# asure

A new SRML-based Substrate node, ready for hacking.

## How to local testnet?
```bash
cargo run -- --base-path /tmp/alice --chain=dev --alice --node-key 0000000000000000000000000000000000000000000000000000000000000001 --validator
```

## How to join the testnet?
```bash
cargo run -- --chain chainspec-testnet.json --name yournodename
```
