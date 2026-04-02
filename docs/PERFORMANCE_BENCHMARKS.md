# Performance Benchmarks

Load testing and performance benchmarking for Claw-Harness.

## Tools

- **k6** - Modern load testing (JavaScript)
- **Locust** - Python load testing
- **hyperfine** - Command-line benchmarking
- **criterion** - Rust benchmarking framework

## k6 Load Tests

### Installation

```bash
# macOS
brew install k6

# Linux
sudo apt install k6

# Docker
docker run --rm grafana/k6
```

### Test Script

```javascript
// benchmarks/k6/claw-harness.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const latency = new Trend('latency_ms');

export const options = {
    stages: [
        { duration: '30s', target: 10 },   // Ramp up to 10 users
        { duration: '1m', target: 50 },    // Ramp up to 50 users
        { duration: '2m', target: 100 },   // Ramp up to 100 users
        { duration: '3m', target: 500 },   // Ramp up to 500 users
        { duration: '2m', target: 500 },   // Stay at 500 users
        { duration: '1m', target: 0 },     // Ramp down
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'],  // 95% of requests should be below 500ms
        errors: ['rate<0.01'],             // Error rate should be < 1%
        latency: ['avg<300'],              // Average latency < 300ms
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
    // Test 1: Run prompt
    const payload = JSON.stringify({
        prompt: 'Explain quantum computing in simple terms',
        tools: [],
    });
    
    const params = {
        headers: { 'Content-Type': 'application/json' },
    };
    
    const response = http.post(`${BASE_URL}/api/v1/run`, payload, params);
    
    const success = check(response, {
        'status is 200': (r) => r.status === 200,
        'has result': (r) => JSON.parse(r.body).result.length > 0,
    });
    
    errorRate.add(!success);
    latency.add(response.timings.duration);
    
    sleep(1);
    
    // Test 2: Health check
    const healthResponse = http.get(`${BASE_URL}/health`);
    check(healthResponse, {
        'health check passes': (r) => r.status === 200,
    });
    
    sleep(0.5);
}
```

### Run k6 Test

```bash
# Local testing
k6 run benchmarks/k6/claw-harness.js

# With custom URL
BASE_URL=http://prod-cluster:8080 k6 run benchmarks/k6/claw-harness.js

# Cloud load testing (1M+ users)
k6 cloud benchmarks/k6/claw-harness.js
```

## Locust Load Tests

### Installation

```bash
pip install locust
```

### Test Script

```python
# benchmarks/locust/locustfile.py

from locust import HttpUser, task, between
import json
import random

class ClawHarnessUser(HttpUser):
    wait_time = between(1, 3)
    
    @task(3)
    def run_prompt(self):
        prompts = [
            "Explain machine learning",
            "Write a haiku about Rust",
            "What is the capital of France?",
            "Debug this Python code: print('hello')",
            "Generate SQL for user table",
        ]
        
        payload = {
            "prompt": random.choice(prompts),
            "tools": []
        }
        
        with self.client.post("/api/v1/run", 
                              json=payload,
                              catch_response=True) as response:
            if response.status_code != 200:
                response.failure("Got wrong status code")
            
            data = response.json()
            if not data.get("result"):
                response.failure("No result in response")
    
    @task(1)
    def list_tools(self):
        self.client.get("/api/v1/tools")
    
    @task(1)
    def health_check(self):
        self.client.get("/health")
```

### Run Locust

```bash
# Web UI
locust -f benchmarks/locust/locustfile.py --host=http://localhost:8080

# Headless mode
locust -f benchmarks/locust/locustfile.py \
    --host=http://localhost:8080 \
    --headless \
    -u 1000 \
    -r 100 \
    --run-time 5m \
    --html=report.html
```

## Rust Benchmarks (Criterion)

### Benchmark Code

```rust
// benches/benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use claw_harness::{ClawHarness, Config};

fn bench_run_prompt(c: &mut Criterion) {
    let config = Config::default();
    let client = ClawHarness::new(config);
    
    c.bench_function("run_prompt_simple", |b| {
        b.iter(|| {
            client.run(black_box("Simple prompt"))
        })
    });
}

fn bench_run_prompt_various_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt_lengths");
    
    for length in [10, 100, 1000, 10000] {
        let prompt = "word ".repeat(length);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(length),
            &prompt,
            |b, prompt| {
                b.iter(|| {
                    let client = ClawHarness::default();
                    client.run(black_box(prompt))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_run_prompt, bench_run_prompt_various_lengths);
criterion_main!(benches);
```

### Run Rust Benchmarks

```bash
cargo bench
```

## Performance Targets

| Metric | Target | Critical |
|--------|--------|----------|
| Single prompt latency (p50) | < 200ms | > 500ms |
| Single prompt latency (p95) | < 500ms | > 1000ms |
| Single prompt latency (p99) | < 1000ms | > 2000ms |
| Throughput (requests/sec) | > 1000 | < 100 |
| Error rate | < 0.1% | > 1% |
| Memory per node | < 500MB | > 2GB |
| CPU per node | < 50% | > 90% |

## Benchmark Results Dashboard

### Grafana Dashboard

Import dashboard ID `12346` for Claw-Harness performance metrics.

### Key Metrics

```promql
# Request rate
rate(http_requests_total{job="claw-harness"}[5m])

# Latency percentiles
histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))

# Error rate
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# Active sessions
claw_sessions_running

# Queue depth
nats_pending_messages{subject="claw.tasks"}
```

## Load Testing Scenarios

### Scenario 1: Burst Traffic

```javascript
// Sudden spike from 0 to 1000 users
stages: [
    { duration: '10s', target: 1000 },
    { duration: '1m', target: 1000 },
    { duration: '10s', target: 0 },
]
```

### Scenario 2: Sustained Load

```javascript
// Constant 500 users for 10 minutes
stages: [
    { duration: '10m', target: 500 },
]
```

### Scenario 3: Ramp Up/Down

```javascript
// Gradual ramp up and down
stages: [
    { duration: '5m', target: 100 },
    { duration: '5m', target: 500 },
    { duration: '5m', target: 100 },
]
```

## Performance Tuning Guide

### 1. Reduce Latency

- Enable connection pooling
- Use HTTP/2
- Implement response caching
- Optimize WASM module loading

### 2. Increase Throughput

- Add more nodes (horizontal scaling)
- Tune NATS batch size
- Optimize TiKV region distribution
- Use async I/O throughout

### 3. Reduce Memory

- Limit concurrent sessions per node
- Tune garbage collection
- Use memory-mapped files for large data
- Implement LRU caching

### 4. Improve Reliability

- Add circuit breakers
- Implement retry with backoff
- Use bulkheads for isolation
- Add health checks and readiness probes
