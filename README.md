**English** | [简体中文](README.zh-CN.md)

<div align="center">

# ⛩️ craft-gate

**One port. Every protocol. Zero hassle.**

A lightweight, async TCP gateway written in Rust that listens on a single port,
sniffs the incoming traffic, and routes it to the right backend — so your
**Minecraft server** and **HTTP services** can share one public port.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/tangge233/craft-gate/releases)

</div>

---

## ✨ Features

- **🚪 Single-port multiplexing** — one listener serves both Minecraft and HTTP traffic
- **🔍 Automatic protocol detection** — sniffs the first bytes of each connection using the [`guess`](https://crates.io/crates/guess) crate (HTTP / TLS / QUIC / web detection)
- **🎯 Smart routing** — HTTP goes to your web service, everything else goes to your Minecraft backend
- **🛡️ Per-IP connection limiting** — protect your backends from connection floods
- **⚡ Pure async** — built on `tokio` with bidirectional `copy`, one task per connection
- **📝 Structured logging** — `tracing`-based, with **daily rolling file logs** + stdout output
- **🧩 TOML configuration** — auto-generates a default config on first run; broken configs are safely backed up (`.bak<timestamp>`) instead of crashing
- **📦 Cross-platform releases** — prebuilt binaries via GitHub Actions (Windows x64/x86/ARM64, Linux x64)

## 🏗️ How it works

craft-gate binds a single TCP listener (e.g. `tcp://0.0.0.0:25565`) and accepts every incoming
connection on it. As soon as a client connects, the gate reads the first 256 bytes and runs them
through a protocol detector powered by the `guess` crate, which recognizes HTTP/TLS/QUIC-style
web traffic. If the sniffed bytes look like web traffic, the connection is relayed to the HTTP
service backend; anything else — for example a Minecraft client — is relayed to the Minecraft
server backend. Before the relay starts, a per-IP connection limiter may reject the client if it
exceeds the configured limit. Each accepted relay runs as its own `tokio` task that bidirectionally
copies data between the client and the chosen backend, so traffic flows freely in both directions
until the connection closes.

## 🚀 Quick start

### From source

```bash
git clone https://github.com/tangge233/craft-gate.git
cd craft-gate
cargo build --release

# run it — a default config will be generated on first launch
./target/release/craft-gate
```

### Download a prebuilt binary

Grab the latest release for your platform from the [Releases page](https://github.com/tangge233/craft-gate/releases).

## ⚙️ Usage

```
Usage: craft-gate [OPTIONS]

Options:
      --config-file <CONFIG_FILE>  Path to the config file [default: craft-gate/config.toml]
      --debug                      Enable debug-level logging
  -h, --help                       Print help
```

## 📄 Configuration

The default config is written to `craft-gate/config.toml` on first run:

```toml
# Address the gate listens on
listen = "tcp://0.0.0.0:25565"

# Per-IP connection limiting
[ip_limit]
enable = false       # set to true to enable
limits = 10          # max concurrent connections per IP

# Minecraft backend
[services.minecraft]
dest = "tcp://127.0.0.1:11451"

# HTTP backend
[services.http]
dest = "tcp://127.0.0.1:8080"
mode = "Proxy"       # "Proxy" or "Redirect"
```

> **Note:** If a config file exists but fails to parse, craft-gate won't crash —
> it backs the file up as `config.bak<timestamp>` and falls back to defaults.

## 📝 Logs

- Logs are written to stdout and to `craft-gate/logs/` as **daily rolling files**.
- Use `--debug` for verbose protocol-detection and connection details.

## 🧪 Tests

```bash
cargo test
```

## 🗺️ Roadmap

- [ ] Wire up the HTTP `Redirect` mode
- [ ] More detection profiles (e.g. plain TCP passthrough rules)
- [ ] Windows ARM64 / macOS binaries in CI

## 📦 Tech stack

| Component | Choice |
|-----------|--------|
| Async runtime | [tokio](https://tokio.rs) |
| Protocol detection | [guess](https://crates.io/crates/guess) |
| Config | TOML via [serde](https://serde.rs) |
| Logging | [tracing](https://crates.io/crates/tracing) + tracing-appender |
| Concurrency primitives | [dashmap](https://crates.io/crates/dashmap) |
| CLI | [clap](https://crates.io/crates/clap) |

## 📜 License

[MIT](LICENSE) © tangge233
