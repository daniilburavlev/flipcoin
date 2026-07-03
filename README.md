# FlipCoin

FlipCoin is a Proof-of-Stake (PoS) blockchain written in Rust. Nodes gossip
blocks, transactions, and votes over libp2p, persist state in RocksDB, and
expose an HTTP API for wallets and peer synchronization.

- **Consensus:** Proof-of-Stake with weighted-random leader selection
- **Accounting:** UTXO model with two output types — `transfer` and `stake`
- **Networking:** libp2p gossipsub (three topics)
- **Storage:** RocksDB (split block headers + txs)
- **API:** axum HTTP server

---

## Architecture

### Workspace layout

All crates live in one Cargo workspace; `node` is the only binary.

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
| `rpc`       | axum server (feature `server`) + reqwest client (feature `client`)          |
| `tx`        | `Tx` struct, UTXO-based construction, fee validation, `UtxoFrom` conversion |
| `utxo`      | `Utxo` / `TxOutput` types, two variants: `Transfer` and `Stake`             |
| `block`     | `Block` struct, SHA256 hash, Merkle roots for txs and validators snapshot   |
| `wallet`    | secp256k1 keypair (reuses libp2p identity), sign/verify, bs58 address       |
| `balance`   | `Balance { address, amount }` — used for stake aggregation                  |
| `pool`      | `MemPool` / `MemPoolLock` types (lives in `state`, defined here)            |
| `common`    | `AppError`, `AppResult`, `BigUint` serde helpers                            |

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

### Consensus (PoS)

Every 12 seconds each node runs `Validator::create_block()`:

1. Collect all current stake holders from the `MemPool`.
2. Compute `roll = hash_to_bigint(last_block.hash)` — SHA256 of the bs58-decoded
   hash, interpreted as a big-endian `BigUint`.
3. Select the validator with a weighted-random walk: the first holder whose
   cumulative stake exceeds `roll % total_stake`.
4. If this node's address matches the selected validator → create and broadcast
   the block.
5. Otherwise → call `Voting::start(height, total_stake)`; when a block arrives
   from the network the node signs a `Vote` and broadcasts it.

### State transitions

- **Genesis:** `node new` stores a synthetic block at height 0 with no
  predecessor. Genesis txs have empty `vin` and `signatures`.
- **add_tx:** validates, checks UTXOs in the `MemPool`, moves consumed UTXOs out,
  adds the tx to the pending pool.
- **add_block:** validates hash + Merkle roots + signature + `prev_hash` chain;
  any txs in the block not already in the pool are processed first; all block txs
  are then converted to new UTXOs via `UtxoFrom`.
- **restore_state** (solo node): replays all stored blocks to rebuild the
  `MemPool` from RocksDB.
- **restore_pool** (joining node): copies `AppState` (txs + UTXOs + stakes maps)
  from a peer via `/api/state`.

### Storage layout (RocksDB column families)

- `BlockStorage`: `BLOCK_CF` (height → BlockData), `BLOCK_BY_HASH` (hash → height)
- `TxStorage`: `TX_CF` (tx_id → Tx), `TX_BY_BLOCK` (height → tx_id list), `TX_BY_WALLET`
- `UtxoStorage`: `UTXO_CF`, `UTXO_BY_BLOCK`, `STAKE_CF`, `STAKE_BY_BLOCK`

Blocks are split: `BlockData` (header fields only) is stored separately from the
txs, which are fetched and recombined on read.

### Encoding conventions

- **Addresses / hashes / signatures:** base58 (`bs58`) strings throughout the API
  and storage.
- **Amounts** (`BigUint`): serialized as decimal strings in JSON via custom serde
  helpers in `common`.
- **Internal serialization** (RocksDB values): `postcard` (binary).

---

## Building

```bash
cargo build            # debug build
cargo build --release  # optimized build
```

## Testing & linting

```bash
cargo test                              # all crates
cargo test -p <crate-name>              # a single crate
cargo test -p <crate-name> <test_name>  # a single test
cargo clippy                            # lint
```

---

## Running a node

### 1. Generate a keypair

Creates a secp256k1 private key at `~/.flipcoin/secret.json`
(`[u8; 32]` as a JSON array).

```bash
cargo run -p node -- keygen
# or a custom path:
cargo run -p node -- keygen --keystore ./secret.json
```

### 2. Initialize the chain from genesis

Stores the genesis block (height 0) into RocksDB. Genesis outputs seed the
initial `stake` and `transfer` balances.

```bash
cargo run -p node -- new --genesis genesis.json
```

Example `genesis.json`:

```json
[
  {
    "tx_id": "H4D71PVocuCEzC7r6AQbeHdBDkJ9uogbbRSYpTUtDLrP",
    "vin": [],
    "vout": [
      { "to": "25VZm3WowtP9cQ2UysLomuSWd4YHXKvdyvAdvv1ekZbfQ", "amount": "100000000000", "vt": "stake" },
      { "to": "25VZm3WowtP9cQ2UysLomuSWd4YHXKvdyvAdvv1ekZbfQ", "amount": "100000000000", "vt": "transfer" }
    ],
    "signatures": [],
    "fee": "0",
    "timestamp": 1009227600
  }
]
```

### 3. Start the node

```bash
cargo run -p node
# or with an explicit config path:
cargo run -p node -- --config ~/.flipcoin/config.toml
```

The node opens its RocksDB storage, joins the p2p network, starts the validator
loop, and serves the HTTP API.

### 4. Send a transfer

Builds a UTXO-based transfer (fee is `amount / 10000`) and submits it to a node.
If `--remote` is omitted, a random peer from the config's `nodes` list is used.

```bash
cargo run -p node -- transfer <address> <amount>
# targeting a specific node:
cargo run -p node -- transfer --remote http://localhost:8080 <address> <amount>
```

---

## Configuration

Config is read from `~/.flipcoin/config.toml` (override with `--config`).
Missing fields fall back to defaults.

```toml
keystore = "secret.json"          # private key path
storage = "data"                  # RocksDB data directory
http_port = 8080                  # HTTP API port      (default 9091)
p2p_port = 8888                   # libp2p port        (default 5413)
nodes = ["http://localhost:9091"] # peers for bootstrap / random transfers
```

### Default file paths

| Path                      | Purpose                                                |
| ------------------------- | ------------------------------------------------------ |
| `~/.flipcoin/config.toml` | Node config (ports, peer URLs, keystore/storage paths) |
| `~/.flipcoin/secret.json` | secp256k1 private key as `[u8; 32]` JSON array         |
| `~/.flipcoin/data/`       | RocksDB data directory                                 |

---

## HTTP API

| Method | Path                           | Description                         |
| ------ | ------------------------------ | ----------------------------------- |
| GET    | `/api/blocks/{height_or_hash}` | Fetch block by height (u64) or hash |
| GET    | `/api/txs`                     | All committed txs                   |
| POST   | `/api/txs`                     | Submit a new tx                     |
| GET    | `/api/txs/{tx_id}`             | Fetch tx by id                      |
| GET    | `/api/info`                    | Node peer info (address, p2p port)  |
| GET    | `/api/state`                   | Full MemPool state (for peer sync)  |
| GET    | `/api/utxos/{address}`         | UTXOs owned by address              |