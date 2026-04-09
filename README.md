# ugly

[![CI](https://github.com/9sx77ssl/ugly/actions/workflows/ci.yml/badge.svg)](https://github.com/9sx77ssl/ugly/actions/workflows/ci.yml)
[![Release](https://github.com/9sx77ssl/ugly/actions/workflows/release.yml/badge.svg)](https://github.com/9sx77ssl/ugly/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/badge/crates.io-v1.0.0-orange.svg)](https://crates.io/crates/ugly)

> High-performance Solana vanity address generator with encrypted storage

## 🚀 Quick Install

### One-liner (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/9sx77ssl/ugly/main/install.sh | sh
```

That's it. The script detects your OS, downloads the pre-built binary, and installs it to `/usr/local/bin`.

### From Source

```bash
git clone https://github.com/9sx77ssl/ugly.git
cd ugly
cargo install --path .
# or
cargo build --release && sudo cp target/release/ugly /usr/local/bin/
```

### Via crates.io

```bash
cargo install ugly
```

## 📖 Usage

### Generate a vanity address

Find a Solana address starting with your pattern:

```bash
ugly generate --pattern moda --threads 8
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --pattern` | Pattern to match (prefix) | *(required)* |
| `-t, --threads` | Number of threads | Auto-detect CPU cores |
| `--cpu-limit` | CPU usage limit % (10–100) | `85` |
| `--max-attempts` | Max attempts before giving up (0 = infinite) | `0` |
| `-o, --output` | Output file for encrypted wallet | `./wallets.enc` |

**CPU limit explained:** `--cpu-limit 50` means the generator will use at most **50% of your CPU capacity**. It adaptively inserts micro-sleeps in the generation loop to keep overall usage around the configured percentage. This prevents your system from becoming unresponsive during generation.

**Output file:** If you don't specify `--output`, wallets are automatically saved to `./wallets.enc` in the current directory when a match is found. The file is encrypted with AES-256-GCM using a key derived from your password via Argon2id.

### Benchmark

Measure how many keys per second your hardware can generate:

```bash
ugly benchmark                    # 10-second test
ugly benchmark --duration 30      # 30-second test
```

This tests 1 to N threads (where N = your CPU core count) and recommends the optimal thread count.

### Decrypt

View wallets stored in an encrypted file:

```bash
ugly decrypt --file wallets.enc
```

You'll be prompted for the password. The private key is displayed in base58 format along with the creation timestamp.

## 📊 Example Output

### Generate

```
🔑 Ugly v1.0.0 — Solana Vanity Address Generator
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Pattern:  moda
  Threads:  8
  CPU:      85%
  Output:   ./wallets.enc
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⠋ [00:02:14] [████████████░░░░░░░░░░░░░░░░░░░░░░░░░░] 5,432,198 tries
  Pattern: "moda" | Threads: 8 | CPU: 85%
  ⚡ 40,523.14 keys/s | ⏱ ETA: ∞

✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓✓ MATCH FOUND!
  Public Key:  modaXk7vH3mQ9pBzR2fGdL5tYnWjCsEuPa
  Saved to:    ./wallets.enc
  Total attempts: 5,432,198
```

### Benchmark

```
⚡ Benchmark: measuring key generation performance
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Duration: 10 seconds
  Testing 1..12 threads
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    1 threads:       12,345.67 keys/sec
    4 threads:       48,234.12 keys/sec
    8 threads:       89,456.33 keys/sec
   12 threads:      121,234.56 keys/sec

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Recommended: 12 threads (121,234.56 keys/sec)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Decrypt

```
🔓 Decrypted Wallet Information
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Wallet #1:
    Public Key:  modaXk7vH3mQ9pBzR2fGdL5tYnWjCsEuPa
    Private Key: 5KjH...64byte_base58_private_key
    Created:     2026-04-09 14:32:01 UTC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Total wallets: 1
```

## 🏗 Architecture

```
src/
├── main.rs            # CLI entry point (clap)
├── cli.rs             # Command definitions
├── config.rs          # Config validation & defaults
├── engine.rs          # Generation core (rayon + atomics)
├── benchmark.rs       # Performance benchmarking
├── decrypt.rs         # Wallet decryption
├── error.rs           # Error types (thiserror)
├── crypto/
│   ├── keygen.rs      # ed25519-dalek + OsRng
│   ├── base58.rs      # Solana base58 encoding
│   └── matcher.rs     # Prefix matching
├── storage/
│   ├── encrypt.rs     # Argon2id + AES-256-GCM
│   └── file.rs        # Encrypted binary I/O
├── ui/
│   ├── progress.rs    # indicatif progress bar
│   └── stats.rs       # Throughput calculator
└── resource/
    └── throttle.rs    # Adaptive CPU throttling
```

## 🔐 Security

| Feature | Implementation |
|---------|---------------|
| **RNG** | `OsRng` — cryptographically secure, OS-provided entropy only |
| **Key memory** | `ZeroizeOnDrop` — private keys overwritten on drop |
| **Encryption** | AES-256-GCM with 12-byte random nonce |
| **Key derivation** | Argon2id — 64 MB memory, 3 iterations |
| **Storage** | Binary format — no plaintext on disk |
| **Password input** | Hidden TTY read via `rpassword` |
| **Logging** | Zero private key data in any output |

### File format

```
Offset  Size   Content
──────  ────   ───────
0       4      Magic: b"UGLY"
4       1      Version: 0x01
5       16     Salt (random)
21      12     Nonce (random)
33      ...    AES-256-GCM ciphertext

Decrypted payload (per wallet):
  32 bytes  — Public key
  64 bytes  — Private key (seed + public)
   8 bytes  — Unix timestamp (LE)
```

## ⚡ Performance

Typical performance on a **12-core / 24-thread** CPU:

| Threads | Keys/sec |
|---------|----------|
| 1       | ~57K     |
| 4       | ~227K    |
| 8       | ~340K    |
| 12      | ~355K    |

Run `ugly benchmark` on your own hardware for accurate numbers.

## 🛠 Build from Source

```bash
# Requirements
# - Rust 1.70+ (edition 2021)

git clone https://github.com/9sx77ssl/ugly.git
cd ugly
cargo build --release
./target/release/ugly --help
```

### Release profile optimizations

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

## 📝 License

[MIT](LICENSE) — do whatever you want with this code.

## ⚠️ Disclaimer

Vanity address generation is probabilistic. Longer patterns take exponentially more time:
- 1 char: ~instant
- 2 chars: seconds
- 3 chars: minutes
- 4 chars: hours to days
- 5+ chars: potentially weeks

Always back up your `wallets.enc` file and remember your password — **there is no way to recover wallets without the password**.
