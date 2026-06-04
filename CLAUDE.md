# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Role

You are a core reviewer only. Do NOT write, modify, or delete any files. Do NOT suggest refactors unless explicitly asked.

## Review Focus

- Logic correctness
- Edge cases and off-by-one
- Null/undefined handling
- Authentication and authorization gaps
- Data validation before persistence
- Incorrect assumptions about external API contracts

## What to Skip

- Code style and formatting (handled by linter)
- Test coverage suggestions
- Performance micro-optimizations unless severe

## Commands

```bash
# Build
cargo build
cargo build --release

# Test all crates
cargo test

# Test a single crate
cargo test -p <crate-name>

# Run a single test
cargo test -p <crate-name> <test_name>

# Lint
cargo clippy

# Run the node
cargo run -p node

# CLI subcommands
cargo run -p node -- keygen                             # generate keypair → ~/.flipcoin/secret.json
cargo run -p node -- new --genesis genesis.json         # initialize genesis block
cargo run -p node -- transfer -a <address> -amount <n>  # send a transfer tx
```

## Architecture

FlipCoin is a Proof-of-Stake blockchain. All crates are in the Cargo workspace; `node` is the only binary.

### Request / data flow

```
CLI (node/src/cli.rs)
  └─ Node (node/src/node.rs)
       ├─ reads Config from ~/.flipcoin/config.toml
       ├─ opens Storage (RocksDB) → wrapped by State
       ├─ creates Chain (chain/src/lib.rs)
       │    ├─ spawns p2p::run() → libp2p gossipsub Runner
       │    ├─ spawns run_validator() — tokio-cron every 12 s
       │    └─ spawns run_state_updater() — tokio::select on 3 mpsc channels
       └─ serves HTTP API (axum, node/src/api.rs)
```

### Key crates

| Crate       | Role                                                                        |
| ----------- | --------------------------------------------------------------------------- |
| `node`      | Binary entrypoint, CLI (clap), HTTP API (axum), config, key loading         |
| `chain`     | Coordinates block production, state updates, and outgoing messages          |
| `state`     | In-memory view: `MemPool` (pending txs + live UTXOs + stakes) + `Storage`   |
| `storage`   | Facade over three RocksDB sub-storages: block, tx, utxo                     |
| `db`        | `async-rocksdb` wrapper with typed column-family access                     |
| `validator` | PoS leader selection; also creates and validates blocks                     |
| `voting`    | Collects peer votes per block height, weighted by stake                     |
| `p2p`       | libp2p gossipsub, three topics: `/block/1.0.0`, `/txs/1.0.0`, `/vote/1.0.0` |
| `rpc`       | Axum server (feature `server`) + reqwest client (feature `client`)          |
| `tx`        | `Tx` struct, UTXO-based construction, fee validation, `UtxoFrom` conversion |
| `utxo`      | `Utxo` / `TxOutput` types, two variants: `Transfer` and `Stake`             |
| `block`     | `Block` struct, SHA256 hash, Merkle roots for txs and validators snapshot   |
| `wallet`    | secp256k1 keypair (reuses libp2p identity), sign/verify, bs58 address       |
| `balance`   | `Balance { address, amount }` — used for stake aggregation                  |
| `pool`      | `MemPool` / `MemPoolLock` types (lives in `state`, defined here)            |
| `common`    | `AppError`, `AppResult`, BigUint serde helpers                              |

### Consensus (PoS)

Every 12 seconds each node runs `Validator::create_block()`:

1. Collect all current stake holders from `MemPool`.
2. Compute `roll = hash_to_bigint(last_block.hash)` (SHA256 of bs58-decoded hash, interpreted as big-endian `BigUint`).
3. Select validator: weighted-random walk — first holder whose cumulative stake exceeds `roll % total_stake`.
4. If this node's address matches → create and broadcast the block.
5. Otherwise → call `Voting::start(height, total_stake)`; when a block arrives from the network the node signs a `Vote` and broadcasts it.

### State transitions

- **Genesis**: `node new` stores a synthetic block at height 0 with no predecessor. Genesis txs have empty `vin` and `signatures`.
- **add_tx**: validates, checks UTXOs in `MemPool`, moves consumed UTXOs out, adds tx to pending pool.
- **add_block**: validates hash + merkle roots + signature + prev_hash chain; for any txs in the block not already in the pool they are processed first; all block txs are then converted to new UTXOs via `UtxoFrom`.
- **restore_state** (solo node): replays all stored blocks to rebuild `MemPool` from RocksDB.
- **restore_pool** (joining node): copies `AppState` (txs + UTXOs + stakes maps) from a peer via `/api/state`.

### Storage layout (RocksDB column families)

`BlockStorage`: `BLOCK_CF` (height → BlockData), `BLOCK_BY_HASH` (hash → height)  
`TxStorage`: `TX_CF` (tx_id → Tx), `TX_BY_BLOCK` (height → tx_id list), `TX_BY_WALLET`  
`UtxoStorage`: `UTXO_CF`, `UTXO_BY_BLOCK`, `STAKE_CF`, `STAKE_BY_BLOCK`

Blocks are split: `BlockData` (header fields only) is stored separately from txs, which are fetched and recombined on read.

### Encoding conventions

- **Addresses / hashes / signatures**: base58 (`bs58`) strings throughout the API and storage.
- **Amounts** (`BigUint`): serialized as decimal strings in JSON via custom serde helpers in `common` (`serialize_biguint` / `deserialize_biguint`).
- **Internal serialization** (RocksDB values): `postcard` (binary).

### Default file paths

| Path                      | Purpose                                                |
| ------------------------- | ------------------------------------------------------ |
| `~/.flipcoin/config.toml` | Node config (ports, peer URLs, keystore/storage paths) |
| `~/.flipcoin/secret.json` | Secp256k1 private key as `[u8; 32]` JSON array         |
| `~/.flipcoin/data/`       | RocksDB data directory                                 |

### HTTP API endpoints

| Method | Path                           | Description                         |
| ------ | ------------------------------ | ----------------------------------- |
| GET    | `/api/blocks/{height_or_hash}` | Fetch block by height (u64) or hash |
| GET    | `/api/txs`                     | All committed txs                   |
| POST   | `/api/txs`                     | Submit a new tx                     |
| GET    | `/api/txs/{tx_id}`             | Fetch tx by id                      |
| GET    | `/api/info`                    | Node peer info (address, p2p port)  |
| GET    | `/api/state`                   | Full MemPool state (for peer sync)  |
| GET    | `/api/utxos/{address}`         | UTXOs owned by address              |
