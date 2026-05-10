# 安装指南

## 前置要求

- Rust 工具链（建议通过 [rustup](https://rustup.rs/) 安装）
- Git

## 编译安装

```bash
# 克隆仓库
git clone https://github.com/Raymer8639/ralr.git
cd ralr

# 编译并安装（需要网络下载依赖）
./install.sh
```

`install.sh` 会依次执行：
1. `cargo build --release` — 以 release 模式编译所有工作区 crate
2. `cargo install --force --path bin/ralr` — 将 `ralr`（VM 运行时）安装到 `~/.cargo/bin/`（`--force` 会覆盖已有的安装）
3. `cargo install --force --path bin/ralr-asm` — 将 `ralr-asm`（汇编器）安装到 `~/.cargo/bin/`（`--force` 会覆盖已有的安装）

安装完成后，`ralr` 和 `ralr-asm` 命令即可在终端中使用。

## 仅编译（不安装）

```bash
cargo build --release
```

编译产物位于 `target/release/ralr` 和 `target/release/ralr_asm`。

## 卸载

```bash
./uninstall.sh
```

或者手动卸载：

```bash
cargo uninstall ralr
cargo uninstall ralr-asm
```

## 开发相关命令

```bash
cargo build                  # 调试模式编译
cargo clippy                 # 运行 linter
cargo fmt                    # 格式化代码
cd tests && cargo test       # 运行测试（tests/ 是独立工作区）
cargo build -p vm_isa        # 仅编译共享库
cargo build -p ralr          # 仅编译 VM 运行时
cargo build -p ralr_asm      # 仅编译汇编器
```
