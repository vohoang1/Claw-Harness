# Kubernetes Deployment for Claw-Harness

## Quick Deploy

```bash
# Install Helm chart
helm install claw-harness ./k8s/helm/claw-harness \
  --namespace claw-system \
  --create-namespace \
  --set replicaCount=3

# Or use kubectl directly
kubectl apply -f k8s/base/
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Ingress (nginx/traefik)         │
└──────────────┬──────────────────────────┘
               │
    ┌──────────▼──────────┐
    │   Load Balancer     │
    │   Service (ClusterIP)│
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │  Claw-Harness Pods  │
    │  (StatefulSet x3)   │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │   NATS Cluster      │
    │   (StatefulSet x3)  │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │   TiKV Cluster      │
    │   (StatefulSet x5)  │
    └─────────────────────┘
```

## Components

### Base Resources (`k8s/base/`)
- Namespace
- ConfigMaps
- Secrets
- ServiceAccounts

### NATS Cluster (`k8s/nats/`)
- StatefulSet (3 replicas)
- Service (headless + client)
- PodDisruptionBudget

### TiKV Cluster (`k8s/tikv/`)
- PD StatefulSet (3 replicas)
- TiKV StatefulSet (5 replicas)
- Services

### Claw-Harness (`k8s/claw/`)
- StatefulSet (configurable replicas)
- Service (ClusterIP + LoadBalancer)
- HorizontalPodAutoscaler
- PodDisruptionBudget

### Monitoring (`k8s/monitoring/`)
- Prometheus StatefulSet
- Grafana Deployment
- ServiceMonitors
- PrometheusRules (alerts)

## Configuration

### values.yaml

```yaml
# Replica counts
replicaCount: 3
natsReplicas: 3
tikvReplicas: 5

# Resources
resources:
  limits:
    cpu: 2
    memory: 4Gi
  requests:
    cpu: 500m
    memory: 1Gi

# Auto-scaling
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
  targetMemoryUtilizationPercentage: 80

# API Keys
apiKeys:
  anthropic: ""  # Set via secret
  openai: ""     # Set via secret

# Ingress
ingress:
  enabled: true
  className: nginx
  hosts:
    - host: claw.example.com
      paths:
        - path: /
          pathType: Prefix
```

## Auto-Scaling

The operator automatically scales based on:

1. **CPU Utilization** - Scale when avg CPU > 80%
2. **Memory Utilization** - Scale when avg memory > 80%
3. **Queue Depth** - Scale based on NATS pending messages
4. **Job Latency** - Scale when p95 latency > threshold

```yaml
# Custom metrics HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: claw-harness
spec:
  metrics:
  - type: Pods
    pods:
      metric:
        name: nats_pending_messages
      target:
        type: AverageValue
        averageValue: 1000
```

## High Availability

### Pod Disruption Budgets

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: claw-harness-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: claw-harness
```

### Anti-Affinity

```yaml
affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
    - labelSelector:
        matchExpressions:
        - key: app
          operator: In
          values:
          - claw-harness
      topologyKey: kubernetes.io/hostname
```

## Monitoring

### Prometheus Metrics

- `claw_sessions_total` - Total sessions
- `claw_tasks_running` - Running tasks
- `claw_tool_latency_seconds` - Tool latency
- `claw_cache_hit_ratio` - Cache efficiency

### Grafana Dashboards

Import dashboard ID `12345` for Claw-Harness overview.

### Alerts

```yaml
groups:
- name: claw-harness
  rules:
  - alert: HighErrorRate
    expr: rate(claw_tool_errors_total[5m]) > 0.1
    for: 5m
    annotations:
      summary: "High tool error rate"
      
  - alert: HighLatency
    expr: histogram_quantile(0.95, rate(claw_tool_latency_seconds_bucket[5m])) > 2
    for: 10m
    annotations:
      summary: "High tool latency"
```

## Backup & Recovery

### TiKV Backup

```bash
# Create backup
tiup br backup full \
  --pd pd.claw-system:2379 \
  --storage "s3://bucket/backups?access-key=xxx&secret-key=yyy"

# Restore from backup
tiup br restore full \
  --pd pd.claw-system:2379 \
  --storage "s3://bucket/backups?access-key=xxx&secret-key=yyy"
```

## Upgrade

```bash
# Rolling update
kubectl rollout restart statefulset claw-harness -n claw-system

# Canary deployment
helm upgrade claw-harness ./k8s/helm/claw-harness \
  --canary \
  --canary-replicas 1
```

## Troubleshooting

```bash
# Check pod status
kubectl get pods -n claw-system

# View logs
kubectl logs -n claw-system claw-harness-0

# Exec into pod
kubectl exec -it -n claw-system claw-harness-0 -- bash

# Check metrics
kubectl top pods -n claw-system
```
