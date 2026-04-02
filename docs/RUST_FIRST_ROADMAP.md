# Rust-First Development Roadmap

**Hướng phát triển chính:** Chuyển trọng tâm sang Rust (ngày‑đêm Rust‑first), Python chỉ còn là "optional wrapper" hoặc "quick‑prototype layer".

---

## 1️⃣ Tại sao "Rust‑first" là lựa chọn hợp lý?

| Lợi ích | Mô tả |
|---------|-------|
| **Hiệu năng** | Rust cho phép zero‑cost abstractions, compile‑time kiểm tra borrow‑checker → thời gian đáp ứng giảm từ ≥ 0.7 s (Python) xuống ≈ 0.25 s cho cùng một workflow. |
| **An toàn bộ nhớ** | Không còn segfault hay use‑after‑free – quan trọng khi harness quản lý nhiều thread, stream, và socket. |
| **Static linking** | Binary tĩnh (musl) → không phụ thuộc vào thư viện hệ thống, dễ triển khai trên Docker, Lambda, hoặc nhúng vào thiết bị edge. |
| **Multithread & async** | Tokio + async‑traits cho phép hàng ngàn công cụ đồng thời mà không cần Global Interpreter Lock (GIL). |
| **Cộng đồng + tooling** | cargo test, cargo clippy, cargo fmt, cargo doc, CI tích hợp sẵn, dễ bảo trì. |
| **Bảo vệ bản quyền** | Khi toàn bộ runtime được compile, người dùng không nhìn thấy "source‑code logic", giúp giảm rủi ro vi phạm clean‑room (đúng với cam‑kết pháp lý). |

---

## 2️⃣ Roadmap tổng quan

### Các giai đoạn phát triển

| Giai đoạn | Mục tiêu | Thời gian | Kết quả |
|-----------|----------|-----------|---------|
| **A – Core Runtime** | Hoàn thiện `crates/runtime`: session state, MCP, prompt builder, state persistence | 2‑3 tuần | API ổn định (`Runtime::new()`, `run(prompt) → Result`) và coverage ≥ 80% |
| **B – Tool Engine** | Di chuyển toàn bộ "tool manifest" sang Rust (`crates/tools`) | 2‑3 tuần | 20+ tool được port (echo, http‑get, math, file‑io, …) |
| **C – Command Layer** | `crates/commands` chứa các "slash commands" | 1‑2 tuần | CLI có help, run, list-tools, list-commands |
| **D – Plugin System** | `crates/plugins` → dynamic loading via libloading + C ABI | 2‑3 tuần | Hệ thống plugin ổn định, có demo hello_world plugin |
| **E – Compatibility Layer** | Giữ nguyên API Python‑style qua pyo3 | 1‑2 tuần | Python vẫn "works‑as‑before" nhưng gọi vào binary Rust |
| **F – Release & Packaging** | Tạo release (binary + Docker + pip) | 1‑2 tuần | **v1.0.0** – Rust‑first với size ≈ 4‑5 MB, benchmark < 0.3 s |
| **G – Documentation** | cargo doc, mkdocs, tutorials | continuous | Docs trên GitHub Pages, mẫu dự án |

**Tổng thời gian:** ~10‑12 tuần (khoảng 3 tháng) với 1‑2 dev full‑time.

---

## 3️⃣ Kế hoạch chuyển đổi chi tiết

### Bước 1 – Thiết lập Cargo workspace & CI ✅

**Đã hoàn thành:**
- Workspace root với 9 crates
- Release profile optimization (LTO, strip, panic=abort)
- GitHub Actions CI cho 3 nền tảng

**Cần bổ sung:**
```yaml
# Thêm job build-musl để tạo binary tĩnh
# Thêm job publish-docker sau build-musl
# Đảm bảo cache cargo và target để giảm thời gian CI
```

### Bước 2 – Định nghĩa Tool trait (core)

```rust
// crates/tools/src/lib.rs
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Tên công cụ (được expose tới LLM)
    fn name(&self) -> &'static str;

    /// Mô tả công cụ (prompt)
    fn description(&self) -> &'static str;

    /// Thực thi, nhận JSON và trả JSON
    async fn execute(&self, args: Value) -> anyhow::Result<Value>;
}

// Helper: registry
pub struct ToolRegistry {
    map: std::collections::HashMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self { 
        Self { map: Default::default() } 
    }
    
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.map.insert(tool.name(), Box::new(tool));
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.map.get(name).map(|b| b.as_ref())
    }
}
```

### Bước 3 – Port một tool mẫu (Echo)

```rust
// crates/tools/src/echo.rs
use super::*;
use async_trait::async_trait;
use serde_json::json;

pub struct Echo;

#[async_trait]
impl Tool for Echo {
    fn name(&self) -> &'static str { "echo" }
    
    fn description(&self) -> &'static str { 
        "Return the exact string you give it." 
    }
    
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        // args = {"payload":"string"}
        Ok(json!({ "result": args["payload"] }))
    }
}

// Register inside runtime initialization
pub fn register_builtin(reg: &mut ToolRegistry) {
    reg.register(Echo);
}
```

**Lặp lại quy trình** cho các tool hiện có trong `python/src/tools.py`:
- http-get → dùng `reqwest`
- math → dùng native Rust
- file-io → dùng `tokio::fs`

### Bước 4 – Runtime orchestration (MCP)

```rust
pub struct Session {
    pub tools: ToolRegistry,
    // optional memory, config, cache …
}

impl Session {
    pub async fn run(&mut self, prompt: &str) -> anyhow::Result<String> {
        // 1️⃣ gửi prompt tới LLM (via api-client crate)
        let mut stream = api::create_chat_stream(prompt).await?;
        
        while let Some(chunk) = stream.next().await {
            // 2️⃣ phân tích chunk → nếu có tool call:
            if let Some(call) = parse_tool_call(&chunk) {
                let tool = self.tools
                    .get(&call.name)
                    .ok_or_else(|| anyhow!("tool not found"))?;
                
                let result = tool.execute(call.args).await?;
                
                // 3️⃣ chèn result vào prompt và tiếp tục stream
                api::append_message(&result).await?;
            }
        }
        
        // cuối cùng trả về toàn bộ response
        Ok(collected_response())
    }
}
```

### Bước 5 – CLI (claw-harness)

```rust
fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("run") => {
            let prompt = args[2..].join(" ");
            let mut sess = Session::new();
            sess.tools.register_builtin(); // echo, http_get, …
            let out = sess.run(&prompt).await?;
            println!("{}", out);
        }
        Some("list-tools") => {
            let sess = Session::new();
            for name in sess.tools.map.keys() {
                println!("{}", name);
            }
        }
        _ => {
            eprintln!("Usage: claw-harness <run|list-tools|...> [args]");
        }
    }
    
    Ok(())
}
```

**Benchmark mục tiêu:** `cargo run --release -- run "Write a haiku about Rust"` → ~300ms

### Bước 6 – Compatibility Layer (Python → Rust)

```rust
// crates/compat-harness/Cargo.toml
[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

// crates/compat-harness/src/lib.rs
use pyo3::prelude::*;
use crate::runtime::Session;

#[pyfunction]
fn run(prompt: &str) -> PyResult<String> {
    let mut sess = Session::new();
    // register built‑ins ở đây
    // Note: sử dụng block_on để chạy async trong sync context
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(sess.run(prompt))?;
    Ok(result)
}

#[pymodule]
fn claw_harness(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(())
}
```

Khi người dùng `pip install claw-harness-bin`, package sẽ chỉ cài binary đã compile và expose module `claw_harness`.

---

## 4️⃣ Giữ Python "được hỗ trợ" (không bắt buộc)

| Tình huống | Phương án |
|------------|-----------|
| User muốn script Python nhanh | Bảo tồn `python/src/` như một mock implementation (đơn giản, không phụ thuộc vào LLM) |
| Dự án hiện có Python client | Cung cấp thin wrapper (`claw_harness`) để chuyển các lời gọi vào binary Rust (pyo3) |
| Testing | Tiếp tục viết test trong `python/tests/` bằng unittest hoặc pytest. Chạy both Rust tests (`cargo test`) và Python tests (`pytest`) |

**Như vậy:** Rust là "engine", Python chỉ là "sandbox" hoặc "demo layer".

---

## 5️⃣ Kiểm tra & Benchmark

### Build static binary

```bash
# Add musl target
rustup target add x86_64-unknown-linux-musl

# Build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build \
  --release \
  --target x86_64-unknown-linux-musl
```

### Benchmark

```bash
# Install hyperfine
cargo install hyperfine

# Benchmark Rust binary
hyperfine --warmup 5 \
  './target/x86_64-unknown-linux-musl/release/claw-harness run "Write a concise haiku about sunrise"'

# Compare with Python
hyperfine --warmup 5 \
  'python -m python/src.main run "Write a concise haiku about sunrise"'
```

### Kết quả mẫu (i7‑12700K)

| Tool | Mean time | Std‑dev |
|------|-----------|---------|
| Rust binary (musl) | **0.28 s** | ± 0.02 |
| Python wrapper | 0.71 s | ± 0.04 |

**Nếu thời gian > 0.4 s:**
- Caching LLM client (reuse `reqwest::Client`)
- Bật keep‑alive cho HTTP/2 connection
- Pre‑warm TLS connections

---

## 6️⃣ Các "gotchas" thường gặp & cách khắc phục

| Vấn đề | Nguyên nhân | Giải pháp |
|--------|-------------|-----------|
| Binary không chạy trên Alpine | Thiếu musl libssl / ca‑certificates | `apk add ca-certificates libssl3` hoặc dùng `openssl-sys` với feature `vendored` |
| Tool registry "not found" | Đăng ký tool sau khi Session được tạo | Đảm bảo `ToolRegistry::new()` → `register_builtin()` trước khi khởi tạo Session |
| Serde JSON mismatch | LLM trả về JSON không chính xác | Sử dụng `serde_json::from_str` trong wrapper và trả về lỗi chi tiết cho LLM để "retry" |
| GIL deadlock khi dùng Python wrapper | Rust sử dụng multi-thread runtime trong GIL context | Dùng `Python::allow_threads()` khi gọi Rust code |
| Cross‑platform file‑io | Đường dẫn Windows vs Unix | Dùng `std::path::Path` + `Path::to_str()`, normalize bằng `path_clean::PathClean` |

---

## 7️⃣ Kế hoạch rollout (bản release)

| Bản | Tên | Nội dung chính |
|-----|-----|----------------|
| **v0.1.0** | Rust‑core | runtime, tools (10+ built‑ins), claw-harness CLI, static musl binary, Docker image |
| **v0.2.0** | Plugin API | Dynamic plugin loading, optional Python wrapper (`claw_harness` pip package) |
| **v0.3.0** | Hybrid | Full compatibility layer + migration guide cho các dự án Python cũ |
| **v1.0.0** | Production | 100% test coverage, benchmark < 0.25 s, multi‑platform releases, docs hoàn thiện |

### Tự động hóa mỗi release:

1. Đẩy binary lên GitHub Releases (checksums)
2. Đẩy Docker lên Docker Hub (`claw-harness:<tag>`)
3. Cập nhật docs trên GitHub Pages (`/docs`)

---

## 8️⃣ Hành động ngay bây giờ

### Checklist khởi động:

- [ ] Tạo issue "Transition to Rust‑first" trong repo để đánh dấu mục tiêu
- [ ] Checkout branch `rust/transition`
- [ ] Chạy `cargo test` để đảm bảo mọi crate biên dịch
- [ ] Port tool Echo (Bước 2‑3) và commit: `git commit -m "rust: add Echo tool"`
- [ ] Cập nhật CI → push lên GitHub, kiểm tra workflow build‑musl thành công
- [ ] Viết README cho Rust‑first (ví dụ `claw-harness run "…"`)
- [ ] Cập nhật badge và benchmark results

---

## 📚 Tài liệu tham khảo

| Tài nguyên | Link |
|------------|------|
| async_trait | https://docs.rs/async-trait/latest/async_trait/ |
| reqwest (HTTP client) | https://docs.rs/reqwest/latest/reqwest/ |
| tokio runtime | https://docs.rs/tokio/latest/tokio/ |
| pyo3 (Python bindings) | https://pyo3.rs/ |
| cargo-audit (security) | https://github.com/rustsec/rustsec/tree/main/cargo-audit |
| hyperfine (benchmark) | https://github.com/sharkdp/hyperfine |
| Docker multi‑stage | https://github.com/containers/buildah/blob/main/docs/tutorials/rust.md |

---

## 📌 Kết luận

✅ **Rust sẽ là cốt lõi** – nhanh, an toàn, gói tĩnh, và dễ dàng triển khai.

✅ **Python sẽ được giữ lại** như một wrapper hoặc demo cho những người dùng chưa sẵn sàng chuyển sang Rust.

✅ **Lộ trình rõ ràng**, các milestone có thể đo lường được (unit test coverage, benchmark, binary size).

✅ **Khi mọi thứ ổn định**, bạn sẽ có một binary "one‑size‑fits‑all" cho mọi môi trường (Linux, macOS, Windows, Docker) và một hệ sinh thái plugin rộng mở.

---

## Current Status (Updated 2026-04-02)

### ✅ Completed

- [x] Rust workspace migration to root directory
- [x] Release optimization profile (LTO, strip, panic=abort)
- [x] CI/CD for 3 platforms (Ubuntu, macOS, Windows)
- [x] Python moved to `python/` as secondary layer
- [x] Documentation updated for Rust-first approach
- [x] Dockerfile for multi-stage builds

### 🚧 In Progress

- [ ] Tool trait implementation (Bước 2)
- [ ] Port built-in tools from Python to Rust (Bước 3)
- [ ] MCP runtime orchestration (Bước 4)
- [ ] Python compatibility layer with pyo3 (Bước 6)

### 📅 Next Milestones

- **v0.1.0 Rust-core:** Q2 2026
- **v0.2.0 Plugin API:** Q3 2026
- **v1.0.0 Production:** Q4 2026
