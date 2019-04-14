# asure

A new SRML-based Substrate node, ready for hacking.

## How to local testnet?

Start first node (Alice)

```bash
cargo run -- --base-path /tmp/alice --chain=local --alice --node-key 0000000000000000000000000000000000000000000000000000000000000001 --validator
```

Then tart second node (Bob)

```bash
cargo run -- --base-path /tmp/bob --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN --chain=local --bob --port 30334 --validator
```

### How to clean local state?
```bash
cargo run -- purge-chain --base-path /tmp/alice --chain=local
cargo run -- purge-chain --base-path /tmp/bob  --chain=local
```

## How to join the testnet?
```bash
cargo run -- --name yournodename
```
