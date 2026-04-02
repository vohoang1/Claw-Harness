# Policy Engine with OPA/Rego

## Overview

The Policy Engine provides fine-grained access control and governance for Claw-Harness operations using Open Policy Agent (OPA) and Rego policies.

## Architecture

```
┌─────────────┐
│   Request   │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Policy Engine  │
│  (OPA/Rego)     │
└──────┬──────────┘
       │
       ▼
┌─────────────┐
│   Allow/    │
│   Deny      │
└─────────────┘
```

## Policy Examples

### 1. Tool Access Control

```rego
package claw.tools

default allow = false

# Allow specific users to use specific tools
allow {
    input.user == "admin"
    input.tool == "shell"
}

allow {
    input.user == "developer"
    input.tool != "shell"  # Developers can't use shell
}

# Allow based on team membership
allow {
    some team in input.teams
    team == "platform-engineering"
    input.tool == "kubernetes"
}
```

### 2. Token Quota Management

```rego
package claw.quota

default allow = false

# Check if user has remaining token quota
allow {
    input.user_tokens_used + input.requested_tokens <= input.user_token_limit
}

# Deny with reason
deny[msg] {
    not allow
    msg := sprintf("Token limit exceeded. Used: %d, Limit: %d", [
        input.user_tokens_used,
        input.user_token_limit
    ])
}
```

### 3. Cost Center Validation

```rego
package claw.billing

default allow = false

# Require valid cost center for expensive operations
allow {
    input.cost_center != ""
    is_valid_cost_center(input.cost_center)
}

is_valid_cost_center(center) {
    some valid_center in input.valid_cost_centers
    center == valid_center
}
```

### 4. Time-Based Access

```rego
package claw.time

default allow = false

# Allow only during business hours
allow {
    time.hour(input.timestamp) >= 9
    time.hour(input.timestamp) <= 18
    time.weekday(input.timestamp) < 5  # Monday-Friday
}
```

### 5. Rate Limiting

```rego
package claw.ratelimit

default allow = false

# Allow if under rate limit
allow {
    input.requests_last_minute < 100
}

deny[msg] {
    not allow
    msg := sprintf("Rate limit exceeded. Requests: %d/100", [input.requests_last_minute])
}
```

## Integration

### Rust Implementation

```rust
use regorus::{Engine, Value};

pub struct PolicyEngine {
    engine: Engine,
}

impl PolicyEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.add_policy_from_file("policies/tools.rego")?;
        Ok(Self { engine })
    }

    pub fn check_tool_access(
        &self,
        user: &str,
        tool: &str,
        teams: &[String],
    ) -> Result<bool, anyhow::Error> {
        let input = serde_json::json!({
            "user": user,
            "tool": tool,
            "teams": teams
        });

        self.engine.set_input(input);
        let result = self.engine.evaluate("data.claw.tools.allow")?;
        
        Ok(result.as_bool().unwrap_or(false))
    }

    pub fn check_quota(
        &self,
        user_tokens: u64,
        requested: u64,
        limit: u64,
    ) -> Result<(bool, Option<String>), anyhow::Error> {
        let input = serde_json::json!({
            "user_tokens_used": user_tokens,
            "requested_tokens": requested,
            "user_token_limit": limit
        });

        self.engine.set_input(input);
        
        let allow = self.engine.evaluate("data.claw.quota.allow")?;
        let deny = self.engine.evaluate("data.claw.quota.deny")?;
        
        let allowed = allow.as_bool().unwrap_or(false);
        let reason = if !allowed {
            deny.as_array().and_then(|arr| arr.first())
                .map(|v| v.as_string().unwrap_or("Access denied".to_string()))
        } else {
            None
        };

        Ok((allowed, reason))
    }
}
```

### Policy Hot-Reload

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PolicyManager {
    engine: Arc<RwLock<Engine>>,
    policy_dir: PathBuf,
}

impl PolicyManager {
    pub async fn watch_for_changes(&self) -> anyhow::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        let mut watcher = watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })?;

        watcher.watch(&self.policy_dir, RecursiveMode::Recursive)?;

        while let Some(event) = rx.recv().await {
            if event.kind.is_modify() {
                self.reload_policies().await?;
                tracing::info!("Policies reloaded");
            }
        }

        Ok(())
    }

    async fn reload_policies(&self) -> anyhow::Result<()> {
        let mut engine = self.engine.write().await;
        engine.clear_policies();
        engine.add_policy_from_file("policies/tools.rego")?;
        engine.add_policy_from_file("policies/quota.rego")?;
        Ok(())
    }
}
```

## Deployment

### OPA Sidecar

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: opa
  namespace: claw-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: opa
  template:
    metadata:
      labels:
        app: opa
    spec:
      containers:
      - name: opa
        image: openpolicyagent/opa:latest
        args:
        - "run"
        - "--server"
        - "--watch"
        - "/policies"
        ports:
        - containerPort: 8181
        volumeMounts:
        - name: policies
          mountPath: /policies
      volumes:
      - name: policies
        configMap:
          name: opa-policies
```

### Policy Bundle

```bash
# Build policy bundle
opa build policies/ -o policy.tar.gz

# Upload to S3
aws s3 cp policy.tar.gz s3://claw-policies/policy.tar.gz

# OPA fetches from S3
opa run --server \
  --bundle s3://claw-policies/policy.tar.gz \
  --set "services.s3.url=https://s3.amazonaws.com"
```

## Testing Policies

```rego
package claw.tools_test

import data.claw.tools

test_admin_can_use_shell {
    tools.allow with input as {"user": "admin", "tool": "shell"}
}

test_developer_cannot_use_shell {
    not tools.allow with input as {"user": "developer", "tool": "shell"}
}
```

```bash
# Run policy tests
opa test policies/ -v
```

## Metrics

```promql
# Policy evaluation count
rate(opa_evals_total[5m])

# Policy evaluation latency
histogram_quantile(0.95, rate(opa_eval_duration_seconds_bucket[5m]))

# Deny rate
rate(opa_deny_total[5m]) / rate(opa_evals_total[5m])
```
