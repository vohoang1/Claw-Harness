# Rust-First Migration Guide

## Tổng quan

Dự án đã được tái cấu trúc để **Rust là ngôn ngữ chính**, Python chỉ đóng vai trò hỗ trợ/tooling.

## Cấu trúc mới

```
Claw-Harness/
├── crates/                 # Rust workspace (PRIMARY)
│   ├── claw-cli/          # Binary CLI chính
│   ├── runtime/           # Session, config, permissions
│   ├── api/               # API clients
│   ├── tools/             # Tool implementations
│   ├── commands/          # Slash commands
│   ├── plugins/           # Plugin system
│   ├── lsp/               # LSP integration
│   ├── server/            # HTTP/SSE server
│   └── compat-harness/    # Compatibility layer
├── python/                 # Python support (SECONDARY)
│   ├── src/
│   └── tests/
├── docs/                   # Documentation
├── Cargo.toml             # Rust workspace config
├── Dockerfile             # Container build
└── README.md              # Main documentation
```

## Build & Run

### Quickstart

```bash
# Build release (tối ưu)
export RUSTFLAGS="-C target-cpu=native"
cargo build --release

# Chạy binary
./target/release/claw --help

# Interactive REPL
./target/release/claw
```

### Development

```bash
# Check only (fast)
cargo check --workspace

# Run tests
cargo test --workspace

# Linting
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt -- --check
```

### Release Build với Optimization

Cargo.toml đã có sẵn profile tối ưu:

```toml
[profile.release]
opt-level = 3
lto = true
panic = "abort"
codegen-units = 1
strip = true
```

Kết quả:
- **Binary size:** ~4-5 MB (stripped)
- **Startup time:** ~50-100ms

## Docker

```bash
# Build image
docker build -t hoang-harness:latest .

# Run container
docker run -it hoang-harness:latest
```

## Python Support

Python chỉ dùng cho:
- Prototype nhanh tính năng mới
- Scripting utilities
- Testing helpers

```bash
cd python
python3 -m src.main summary
python3 -m unittest discover -s tests -v
```

## CI/CD

GitHub Actions workflow bao gồm:
- **Rust CI:** fmt, check, clippy, test, release build (Ubuntu, macOS, Windows)
- **Python Tests:** Unit tests (Ubuntu)

## Migration từ cấu trúc cũ

Nếu bạn đang dùng cấu trúc cũ (`rust/` folder):

```bash
# Old
cd rust && cargo build --release

# New
cargo build --release
```

Binary location thay đổi:
- **Old:** `rust/target/release/claw`
- **New:** `target/release/claw`

## Troubleshooting

### Windows: Unix-specific tests fail

Một số tests sử dụng `std::os::unix` sẽ bị skip trên Windows. Đây là expected behavior.

### Build chậm lần đầu

Cargo cần compile dependencies. Sử dụng cache:

```bash
# Cargo cache giúp build lại nhanh hơn
export CARGO_INCREMENTAL=1
```

### OpenSSL errors (Linux)

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# Fedora/RHEL
sudo dnf install openssl-devel
```

## Performance Tips

1. **Build với target-cpu native:**
   ```bash
   export RUSTFLAGS="-C target-cpu=native"
   cargo build --release
   ```

2. **Sử dụng mold linker (Linux):**
   ```bash
   sudo apt install mold
   export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
   ```

3. **Cache cargo dependencies:**
   ```bash
   # Trong CI, cache ~/.cargo/registry và target/
   ```

## Next Steps

- [ ] Publish crates.io packages
- [ ] Add release packaging workflow
- [ ] Windows MSI installer
- [ ] Homebrew formula for macOS
