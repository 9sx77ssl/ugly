# ugly

[![CI](https://github.com/9sx77ssl/ugly/actions/workflows/ci.yml/badge.svg)](https://github.com/9sx77ssl/ugly/actions/workflows/ci.yml)
[![Release](https://github.com/9sx77ssl/ugly/actions/workflows/release.yml/badge.svg)](https://github.com/9sx77ssl/ugly/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ugly.svg)](https://crates.io/crates/ugly)
[![Downloads](https://img.shields.io/crates/d/ugly.svg)](https://crates.io/crates/ugly)

High-performance Solana vanity address generator with encrypted wallet storage.

---

## Install

### One-liner (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/9sx77ssl/ugly/main/install.sh | sh
```

Downloads a pre-built binary from GitHub Releases and installs it to `/usr/local/bin`. No Rust toolchain required.

### Via cargo

```bash
cargo install ugly
```

### From source

```bash
git clone https://github.com/9sx77ssl/ugly.git
cd ugly
cargo build --release
sudo cp target/release/ugly /usr/local/bin/
```

---

## Quick Start

### Generate an address starting with a pattern

```bash
ugly generate --pattern moda --threads 8
```

Generates keypairs in parallel, checks if the base58-encoded public key starts with `moda`. When found, encrypts and saves to a file.

### Benchmark your hardware

```bash
ugly benchmark
```

Tests 1 to N threads and reports keys/sec for each, recommending the optimal thread count.

### Decrypt saved wallets

```bash
ugly decrypt --file wallets.enc
```

Reads the encrypted file, prompts for password, displays public/private keys.

---

## Commands

### `ugly generate`

Generates vanity keypairs until a match is found.

```bash
ugly generate --pattern abc --threads 4 --cpu-limit 80 --output my_wallets.enc
```

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --pattern` | **Required.** Prefix to match in the address | - |
| `-t, --threads` | Number of threads to use | Auto-detect CPU cores |
| `--cpu-limit` | CPU usage cap in percent (10-100) | 85 |
| `--max-attempts` | Stop after this many attempts (0 = unlimited) | 0 |
| `-o, --output` | Encrypted wallet output file path | `./wallets.enc` |

**CPU limit**: `--cpu-limit 50` keeps CPU usage around 50% using adaptive micro-sleeps. Prevents system slowdown during generation.

**Match handling**: Uses atomic compare-and-swap to ensure only one thread processes a match, even if multiple threads find it simultaneously.

### `ugly benchmark`

Measures key generation throughput across thread counts.

```bash
ugly benchmark                # 10 seconds
ugly benchmark --duration 30  # 30 seconds
```

Outputs a table of threads vs keys/sec with a recommendation for the best thread count.

### `ugly decrypt`

Decrypts and displays wallet contents.

```bash
ugly decrypt --file wallets.enc
```

---

## Security

| Aspect | Implementation |
|--------|---------------|
| RNG | `OsRng` -- OS-provided entropy only, no seeds |
| Key memory | `ZeroizeOnDrop` -- private keys overwritten on drop |
| File encryption | AES-256-GCM with random nonce |
| Key derivation | Argon2id -- 64 MB memory, 3 iterations |
| Disk storage | Binary format only, no plaintext |
| Password input | Hidden TTY read via `rpassword` |
| Logging | Private keys never logged |

### File format

```
Offset  Size   Content
------  ----   -------
0       4      Magic: b"UGLY"
4       1      Version: 0x01
5       16     Salt (random)
21      12     Nonce (random)
33      ...    AES-256-GCM ciphertext
```

Each decrypted wallet entry (104 bytes):
- 32 bytes -- public key (the Solana address)
- 64 bytes -- private key (seed + public key bytes)
- 8 bytes -- Unix timestamp (little-endian)

---

## Performance

Typical results on a 12-core CPU:

| Threads | Keys/sec |
|---------|----------|
| 1       | ~55K     |
| 4       | ~199K    |
| 8       | ~279K    |
| 12      | ~321K    |

Run `ugly benchmark` for accurate numbers on your hardware.

---

## How long does it take?

Vanity generation is probabilistic. Longer patterns take exponentially more time:

| Pattern length | Avg attempts | At ~300K keys/sec |
|----------------|--------------|-------------------|
| 1 char         | ~4           | Instant           |
| 2 chars        | ~190         | Instant           |
| 3 chars        | ~9,000       | < 1 sec           |
| 4 chars        | ~430,000     | ~1 sec            |
| 5 chars        | ~20M         | ~1 min            |
| 6 chars        | ~1B          | ~55 min           |
| 7 chars        | ~48B         | ~2 days           |
| 8 chars        | ~2.3T        | ~3 months         |

These are averages. You might get lucky in seconds or run for weeks.

---

## Architecture

```
src/
├── main.rs            # Entry point, command dispatch
├── cli.rs             # Clap CLI definitions
├── config.rs          # Config validation
├── engine.rs          # Core generation (rayon + atomics + CAS)
├── benchmark.rs       # Performance benchmarking
├── decrypt.rs         # Wallet decryption
├── error.rs           # Error types
├── crypto/
│   ├── keygen.rs      # ed25519 keypair generation + ZeroizeOnDrop
│   ├── base58.rs      # Solana base58 encoding
│   └── matcher.rs     # Prefix matching
├── storage/
│   ├── encrypt.rs     # Argon2id + AES-256-GCM
│   └── file.rs        # Encrypted binary file I/O
├── ui/
│   └── progress.rs    # Terminal progress bar
└── resource/
    └── throttle.rs    # Adaptive CPU throttling
```

### Technical details

- **Rayon** -- work-stealing thread pool with automatic load balancing
- **AtomicU64 + Relaxed ordering** -- lock-free attempt counters
- **Compare-and-swap (CAS)** -- guarantees single-thread match handling
- **Batch size 4096** -- balances throughput with responsiveness
- **Pre-allocated buffers** -- zero allocations in the hot loop
- **ZeroizeOnDrop** -- private keys securely wiped on drop

---

## Building from Source

```bash
# Requires Rust 1.70+
git clone https://github.com/9sx77ssl/ugly.git
cd ugly
cargo build --release
./target/release/ugly --help
```

### Release profile

```toml
[profile.release]
opt-level = 3       # Full optimizations
lto = true          # Link-time optimization
codegen-units = 1   # Single compilation unit
panic = "abort"     # No unwinding, smaller binary
```

---

## Important Notes

1. **Remember your password.** There is no password reset. Wallets cannot be recovered without it.
2. **Back up `wallets.enc`.** The file is fully self-contained (includes salt, nonce, and encrypted data).
3. **Long patterns take time.** 8 characters on a typical CPU = months. Plan accordingly.
4. **Never log private keys.** The tool doesn't, but be careful if you modify the code.

---

## License

[MIT](LICENSE)

---

## Links

- [Issues](https://github.com/9sx77ssl/ugly/issues)
- [Crates.io](https://crates.io/crates/ugly)
