# Solana Storage Program

一个基于 Solana 的链上数据存储器项目。

## 关于依赖版本锁定

本项目明确指定了部分依赖包的版本，原因如下：

### 为什么需要锁定版本？

Solana SBF（Solana Bytecode Format）工具链当前版本（**v1.84**）尚不支持 Rust Edition 2024。如果使用最新版本的某些依赖包，会因要求 Edition 2024 而导致编译失败。

### 具体锁定说明

| 依赖包 | 锁定版本 | 原因 |
|--------|----------|------|
| `blake3` | `=1.5.5` | 保留 `digest` 特性支持，同时避免 Edition 2024 要求 |
| `constant_time_eq` | `=0.3.1` | 防止新版 Edition 2024 引发的 manifest 解析错误 |
| `base64ct` | `=1.7.3` | 确保恒定时间 Base64 编解码的一致性 |

> **注意**：当 Solana SBF 工具链支持 Edition 2024 后，这些版本限制可以移除。

## 环境要求

- Rust 工具链（stable）
- Solana CLI (stable)

安装依赖项请参考：[Solana入门安装依赖项](https://solana.com/zh/docs/intro/installation/dependencies)

## 运行项目

### 本地开发

1. **克隆仓库**

```bash
git clone <repository-url>
cd solana-storage
```

2. **构建 BPF 程序**

```bash
cargo build-sbf
```

3. **更改Solana CLI集群为开发环境**

```bash
solana config set --url devnet
```

4. **部署**

```bash
solana program deploy target/deploy/solana_storage.so
```

## 许可证

MIT
