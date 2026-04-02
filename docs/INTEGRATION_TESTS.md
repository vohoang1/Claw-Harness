# Integration Tests

End-to-end testing suite for Claw-Harness distributed platform.

## Test Structure

```
tests/
├── integration/
│   ├── mod.rs
│   ├── test_distributed_runtime.rs
│   ├── test_wasm_sandbox.rs
│   ├── test_tool_store.rs
│   ├── test_grpc_api.rs
│   └── test_python_sdk.py
├── e2e/
│   ├── mod.rs
│   ├── test_full_workflow.rs
│   └── test_multi_node.rs
└── fixtures/
    ├── test_packages/
    └── test_wasm/
```

## Running Tests

```bash
# Run all integration tests
cargo test --test integration

# Run specific test
cargo test --test integration test_distributed_runtime

# Run with coverage
cargo tarpaulin --test integration --out Html

# Run e2e tests (requires Docker)
./scripts/run-e2e-tests.sh
```

## Test Examples

### Distributed Runtime Test

```rust
// tests/integration/test_distributed_runtime.rs

use distributed_runtime::{Job, TaskQueue, StateStore, ToolTask};
use uuid::Uuid;

#[tokio::test]
async fn test_job_lifecycle() {
    // Setup
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let tikv_pd = std::env::var("TIKV_PD")
        .unwrap_or_else(|_| "127.0.0.1:2379".to_string());

    let queue = TaskQueue::new(&nats_url).await.unwrap();
    let state = StateStore::new(vec![tikv_pd]).await.unwrap();

    // Create job
    let session_id = Uuid::new_v4();
    let mut job = Job::new(session_id, "Test prompt".to_string());
    
    // Store job
    state.put_job(&job).await.unwrap();
    
    // Retrieve job
    let retrieved = state.get_job(job.id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, job.id);
    assert_eq!(retrieved.prompt, job.prompt);

    // Publish task
    let task = ToolTask::new(
        job.id,
        session_id,
        distributed_runtime::ToolCall {
            name: "echo".to_string(),
            args: serde_json::json!({"text": "hello"}),
            call_id: "call_123".to_string(),
        }
    );
    
    queue.publish(&task).await.unwrap();

    // Cleanup
    state.delete_job(job.id).await.unwrap();
}

#[tokio::test]
async fn test_multi_node_job_distribution() {
    // Simulate 3 nodes processing jobs
    let nodes = vec!["node-1", "node-2", "node-3"];
    
    for node in nodes {
        // Each node should be able to pick up and process jobs
        // Implement work-stealing logic verification
        todo!("Implement multi-node test");
    }
}
```

### WASM Sandbox Test

```rust
// tests/integration/test_wasm_sandbox.rs

use sandbox::WasmSandbox;
use std::path::Path;

#[tokio::test]
async fn test_wasm_echo() {
    let sandbox = WasmSandbox::new().unwrap();
    
    // Create test WASM module (echo program)
    let wasm_path = Path::new("tests/fixtures/test_wasm/echo.wasm");
    
    let result = sandbox.execute(
        wasm_path,
        &["hello".to_string()],
        &[],
    ).await.unwrap();
    
    assert_eq!(result["echo"], "hello");
}

#[tokio::test]
async fn test_wasm_sandbox_isolation() {
    let sandbox = WasmSandbox::new().unwrap();
    
    // Try to access filesystem (should be blocked)
    let wasm_path = Path::new("tests/fixtures/test_wasm/fs_access.wasm");
    
    let result = sandbox.execute(
        wasm_path,
        &[],
        &[],
    ).await;
    
    // Should fail or return limited access
    assert!(result.is_err() || result.unwrap()["error"].is_string());
}
```

### gRPC API Test

```rust
// tests/integration/test_grpc_api.rs

use grpc_api::generated::*;
use tonic::Request;

#[tokio::test]
async fn test_grpc_health_check() {
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();
    
    let mut client = claw_service_client::ClawServiceClient::new(channel);
    
    let response = client
        .health_check(Request::new(HealthCheckRequest {}))
        .await
        .unwrap();
    
    let health = response.into_inner();
    assert!(health.healthy);
    assert!(!health.version.is_empty());
}

#[tokio::test]
async fn test_grpc_run() {
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();
    
    let mut client = claw_service_client::ClawServiceClient::new(channel);
    
    let request = Request::new(RunRequest {
        prompt: "Test prompt".to_string(),
        tools: vec![],
        session_id: None,
        model: None,
    });
    
    let response = client.run(request).await.unwrap();
    let result = response.into_inner();
    
    assert!(!result.result.is_empty());
    assert!(result.usage.is_some());
}
```

### Python SDK Test

```python
# tests/integration/test_python_sdk.py

import pytest
import asyncio
from claw_harness import ClawHarness, AsyncClawHarness

def test_sync_client():
    """Test synchronous client"""
    with ClawHarness(node_url="http://localhost:8080") as client:
        response = client.run("Test prompt")
        assert isinstance(response, str)
        assert len(response) > 0

@pytest.mark.asyncio
async def test_async_client():
    """Test asynchronous client"""
    async with AsyncClawHarness() as client:
        response = await client.run("Test prompt")
        assert isinstance(response, str)
        assert len(response) > 0

@pytest.mark.asyncio
async def test_session_management():
    """Test session creation and retrieval"""
    async with AsyncClawHarness() as client:
        session = await client.create_session()
        assert session.id is not None
        
        retrieved = await client.get_session(session.id)
        assert retrieved.id == session.id

@pytest.mark.asyncio
async def test_tool_listing():
    """Test listing available tools"""
    async with AsyncClawHarness() as client:
        tools = await client.list_tools()
        assert isinstance(tools, list)
        # Should have at least built-in tools
        assert len(tools) > 0
```

## Fixtures

### Test WASM Module

```rust
// tests/fixtures/test_wasm/echo.rs
// Compile with: cargo build --target wasm32-wasi --release

use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = args.get(1).map(|s| s.as_str()).unwrap_or("");
    
    let result = serde_json::json!({
        "echo": input
    });
    
    let mut file = fs::File::create("/tmp/result.json").unwrap();
    writeln!(file, "{}", result).unwrap();
}
```

## CI Integration

```yaml
# .github/workflows/integration-tests.yml

name: Integration Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  integration:
    runs-on: ubuntu-latest
    
    services:
      nats:
        image: nats:latest
        ports:
          - 4222:4222
      
      tikv:
        image: pingcap/tikv:latest
        ports:
          - 20160:20160
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run integration tests
        run: cargo test --test integration
        env:
          NATS_URL: nats://localhost:4222
          TIKV_PD: localhost:2379
      
      - name: Run e2e tests
        run: ./scripts/run-e2e-tests.sh
```
