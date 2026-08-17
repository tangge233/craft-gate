<div align="center">

# ⛩️ craft-gate

**一个端口，多种协议，零负担。**

一个用 Rust 编写的轻量级异步 TCP 网关。只需监听一个端口，即可自动识别流入的流量协议，并转发到对应的后端服务——让你的 **Minecraft 服务器**和 **HTTP 服务**共享同一个公网端口。

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/tangge233/craft-gate/releases)

</div>

---

## ✨ 特性

- **🚪 单端口多路复用** — 一个监听端口同时服务 Minecraft 与 HTTP 流量
- **🔍 自动协议识别** — 基于 [`guess`](https://crates.io/crates/guess) crate 嗅探连接的前几个字节（HTTP / TLS / QUIC / web 检测）
- **🎯 智能路由** — HTTP 流量转发到 Web 服务，其余流量转发到 Minecraft 后端
- **🛡️ 单 IP 连接数限制** — 保护后端服务免受连接洪泛攻击
- **⚡ 纯异步** — 基于 `tokio`，双向 `copy` 转发，每个连接一个任务
- **📝 结构化日志** — 基于 `tracing`，支持**按天滚动文件日志** + 标准输出
- **🧩 TOML 配置** — 首次运行自动生成默认配置；配置损坏时自动备份为 `.bak<时间戳>` 而不是直接崩溃
- **📦 跨平台发布** — GitHub Actions 自动构建（Windows x64/x86/ARM64、Linux x64）

## 🏗️ 工作原理

craft-gate 只绑定一个 TCP 监听端口（如 `tcp://0.0.0.0:25565`），所有连接都从该端口进入。客户端一连接，网关就会读取前 256 个字节，并交给基于 `guess` crate 的协议检测器识别——它可以识别 HTTP / TLS / QUIC 等 Web 类流量。如果嗅探到的字节看起来是 Web 流量，连接就被转发到 HTTP 服务后端；其余流量（例如 Minecraft 客户端）则转发到 Minecraft 服务器后端。在转发开始前，单 IP 连接数限制器会检查该客户端是否超过配置的上限，超限则直接拒绝。每个被接受的连接都会作为独立的 `tokio` 任务运行，在客户端与选定的后端之间双向拷贝数据，双向流量畅通无阻，直到连接关闭为止。

## 🚀 快速开始

### 从源码构建

```bash
git clone https://github.com/tangge233/craft-gate.git
cd craft-gate
cargo build --release

# 运行 —— 首次启动会自动生成默认配置
./target/release/craft-gate
```

### 下载预编译二进制

在 [Releases 页面](https://github.com/tangge233/craft-gate/releases) 获取适合你平台的版本。

## ⚙️ 使用方式

```
Usage: craft-gate [OPTIONS]

Options:
      --config-file <CONFIG_FILE>  配置文件路径 [默认: craft-gate/config.toml]
      --debug                      开启 debug 级别日志
  -h, --help                       打印帮助信息
```

## 📄 配置说明

首次运行会在 `craft-gate/config.toml` 生成默认配置：

```toml
# 网关监听地址
listen = "tcp://0.0.0.0:25565"

# 单 IP 连接数限制
[ip_limit]
enable = false       # 设为 true 开启
limits = 10          # 每个 IP 的最大并发连接数

# Minecraft 后端
[services.minecraft]
dest = "tcp://127.0.0.1:11451"

# HTTP 后端
[services.http]
dest = "tcp://127.0.0.1:8080"
mode = "Proxy"       # "Proxy" 或 "Redirect"
```

> **提示：** 如果配置文件存在但解析失败，craft-gate 不会崩溃——它会将原文件备份为 `config.bak<时间戳>` 并回退到默认配置。

## 📝 日志

- 日志同时输出到标准输出和 `craft-gate/logs/`，按天滚动。
- 使用 `--debug` 可查看详细的协议检测与连接信息。

## 🧪 测试

```bash
cargo test
```

## 🗺️ 路线图

- [ ] 实现 HTTP `Redirect` 模式
- [ ] 更多检测配置（如纯 TCP 透传规则）
- [ ] CI 增加 Windows ARM64 / macOS 构建产物

## 📦 技术栈

| 组件 | 选型 |
|-----------|--------|
| 异步运行时 | [tokio](https://tokio.rs) |
| 协议检测 | [guess](https://crates.io/crates/guess) |
| 配置解析 | [serde](https://serde.rs) + TOML |
| 日志 | [tracing](https://crates.io/crates/tracing) + tracing-appender |
| 并发容器 | [dashmap](https://crates.io/crates/dashmap) |
| 命令行 | [clap](https://crates.io/crates/clap) |

## 📜 许可证

[MIT](LICENSE) © tangge233
