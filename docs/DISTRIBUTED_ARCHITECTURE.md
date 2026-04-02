# Distributed Claw-Harness Architecture

## Overview

This document describes the distributed architecture for Claw-Harness, transforming it from a single-node CLI into a **production-grade, distributed AI agent orchestration platform**.

## Architecture Layers

```
┌───────────────────────────────────────────────────────────────┐
│                     USER‑FACING INTERFACE                     │
│  - CLI (claw-harness)                                         │
│  - Python SDK (claw_harness)                                  │
│  - REST / gRPC API                                            │
│  - Web UI (React + Wasm)                                      │
└───────────────────────┬───────────────────────────────────────┘
                        │
        ┌───────────────▼───────────────┐
        │  DISTRIBUTED RUNTIME LAYER      │
        │  (Rust core)                   │
        │  • Scheduler & Dispatcher      │
        │  • Multi‑node Task Queue       │
        │  • Distributed State Store      │
        │  • Fault‑tolerant Replication  │
        │  • Built‑in Metrics & Tracing   │
        └───────┬───────────────┬───────┘
                │               │
   ┌───────────────┐   ┌───────────────┐
   │   PLUGIN SAND‑ │   │   TOOL STORE   │
   │   BOX (WASI)   │   │   (NPM‑like)   │
   └───────────────┘   └───────────────┘
```

## New Crates

### 1. `distributed-runtime`

**Purpose:** Distributed job scheduling, task queue, and state management.

**Dependencies:**
- NATS (async messaging)
- TiKV (distributed KV store)
- Tokio (async runtime)

**Key Components:**
- `Job` - Represents a conversation session
- `ToolTask` - Represents a tool execution request
- `TaskQueue` - NATS-based publish/subscribe queue
- `StateStore` - TiKV-based distributed state

**Example Usage:**
```rust
use distributed_runtime::{Job, TaskQueue, StateStore};

// Connect to NATS
let queue = TaskQueue::new("nats://localhost:4222").await?;

// Connect to TiKV
let state = StateStore::new(vec!["127.0.0.1:2379".to_string()]).await?;

// Create and store a job
let job = Job::new(session_id, prompt);
state.put_job(&job).await?;

// Publish task to queue
queue.publish(&task).await?;
```

### 2. `sandbox`

**Purpose:** Secure plugin execution using Wasmtime and WASI.

**Dependencies:**
- wasmtime (WASM runtime)
- wasmtime-wasi (WASI support)

**Key Components:**
- `WasmSandbox` - Executes WASM modules with WASI support
- Secure isolation with resource limits

**Example Usage:**
```rust
use sandbox::WasmSandbox;

let sandbox = WasmSandbox::new()?;
let result = sandbox.execute(
    Path::new("plugin.wasm"),
    &["arg1".to_string()],
    &[("ENV_VAR".to_string(), "value".to_string())],
).await?;
```

### 3. `metrics`

**Purpose:** Observability with Prometheus metrics and OpenTelemetry tracing.

**Dependencies:**
- prometheus (metrics)
- opentelemetry (tracing)
- tracing-opentelemetry (integration)

**Key Metrics:**
- `claw_sessions_total` - Total sessions created
- `claw_tasks_running` - Current running tasks
- `claw_tool_calls_total` - Total tool calls
- `claw_tool_latency_seconds` - Tool execution latency
- `claw_cache_hits_total` - Cache hits
- `claw_llm_tokens_total` - LLM token usage

**Example Usage:**
```rust
use metrics::{init_metrics, init_tracing, gather_metrics};

// Initialize at startup
init_metrics()?;
init_tracing("claw-harness-node-1", "http://localhost:4317")?;

// Expose /metrics endpoint
#[axum::routing::get("/metrics")]
async fn metrics_handler() -> String {
    gather_metrics()
}
```

## Distributed Flow

### 1. Job Creation
```
User Prompt → CLI → Create Job → Store in TiKV → Publish to NATS Queue
```

### 2. Task Execution
```
Node subscribes to NATS → Pull Task → Execute in WASM Sandbox → 
Store Result in TiKV → Publish Result to NATS
```

### 3. State Management
```
Job State → TiKV (replicated across nodes)
Session State → TiKV (keyed by session_id)
Tool Cache → TiKV (with TTL)
```

## Deployment

### Single Node (Development)
```bash
# Just run the CLI
cargo run --bin claw -- "Your prompt"
```

### Multi-Node (Production)
```bash
# Start NATS
docker run -d --name nats -p 4222:4222 nats:latest

# Start TiKV (using docker-compose)
docker-compose up -d tikv pd

# Start multiple Claw-Harness nodes
cargo run --bin claw-node -- --id node-1 --nats nats://localhost:4222
cargo run --bin claw-node -- --id node-2 --nats nats://localhost:4222
```

### Kubernetes (Scale)
```yaml
# See k8s/ directory for Helm charts and operators
apiVersion: claw-harness.io/v1
kind: ClawCluster
metadata:
  name: production
spec:
  nodeReplicas: 3
  natsReplicas: 3
  tikvReplicas: 5
```

## Security

### WASM Sandbox Isolation
- No direct filesystem access
- No network access (unless explicitly allowed)
- CPU/Memory limits via WASI
- Seccomp profiles for additional isolation

### Authentication
- OAuth 2.0 for user authentication
- API keys for service-to-service communication
- mTLS for inter-node communication

## Observability

### Metrics Dashboard
```promql
# Sessions per minute
rate(claw_sessions_total[1m])

# Average tool latency
histogram_quantile(0.95, rate(claw_tool_latency_seconds_bucket[5m]))

# Cache hit ratio
claw_cache_hits_total / (claw_cache_hits_total + claw_cache_misses_total)
```

### Distributed Tracing
Each request gets a unique trace ID that flows through:
1. CLI/API entry point
2. Job scheduler
3. Task executor
4. WASM sandbox
5. LLM client
6. Result aggregation

View traces in Jaeger/Zipkin UI.

## Next Steps

1. **Phase 5:** Tool Store CLI commands
2. **Phase 6:** Docker Compose setup
3. **Phase 7:** Python SDK with gRPC
4. **Phase 8:** Documentation & examples

## References

- [NATS Documentation](https://docs.nats.io/)
- [TiKV Documentation](https://tikv.org/docs/)
- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
