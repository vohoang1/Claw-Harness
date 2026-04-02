# Helm Chart for Claw-Harness

Production-ready Kubernetes deployment.

## Chart Structure

```
helm/claw-harness/
├── Chart.yaml
├── values.yaml
├── values.production.yaml
├── templates/
│   ├── _helpers.tpl
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   ├── serviceaccount.yaml
│   ├── nats/
│   │   ├── statefulset.yaml
│   │   └── service.yaml
│   ├── tikv/
│   │   ├── pd-statefulset.yaml
│   │   ├── tikv-statefulset.yaml
│   │   └── services.yaml
│   ├── claw/
│   │   ├── statefulset.yaml
│   │   ├── service.yaml
│   │   ├── hpa.yaml
│   │   └── pdb.yaml
│   ├── monitoring/
│   │   ├── prometheus.yaml
│   │   └── grafana.yaml
│   └── ingress.yaml
└── charts/  # Subcharts
```

## Chart.yaml

```yaml
apiVersion: v2
name: claw-harness
description: Distributed AI Agent Orchestration Platform
type: application
version: 0.2.0
appVersion: "0.2.0"
keywords:
  - ai
  - llm
  - agent
  - distributed
  - harness
home: https://github.com/vohoang1/Claw-Harness
sources:
  - https://github.com/vohoang1/Claw-Harness
maintainers:
  - name: Claw-Harness Team
    email: maintainers@claw-harness.io
dependencies:
  - name: nats
    version: 1.2.0
    repository: https://nats-io.github.io/k8s/helm/charts/
  - name: tikv
    version: 0.1.0
    repository: https://tikv.github.io/charts/
  - name: prometheus
    version: 15.0.0
    repository: https://prometheus-community.github.io/helm-charts
  - name: grafana
    version: 6.0.0
    repository: https://grafana.github.io/helm-charts/
```

## values.yaml

```yaml
# Default values for claw-harness

# Global settings
global:
  registry: docker.io
  imagePullPolicy: IfNotPresent
  storageClass: standard

# Claw-Harness nodes
claw:
  replicaCount: 3
  image:
    repository: claw-harness/node
    tag: latest
  resources:
    limits:
      cpu: 2
      memory: 4Gi
    requests:
      cpu: 500m
      memory: 1Gi
  autoscaling:
    enabled: true
    minReplicas: 3
    maxReplicas: 10
    targetCPUUtilizationPercentage: 80
    targetMemoryUtilizationPercentage: 80
  service:
    type: ClusterIP
    port: 8080
  ingress:
    enabled: true
    className: nginx
    annotations:
      nginx.ingress.kubernetes.io/rate-limit: "100"
      nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    hosts:
      - host: claw.example.com
        paths:
          - path: /
            pathType: Prefix
    tls:
      - secretName: claw-tls
        hosts:
          - claw.example.com

# NATS configuration
nats:
  enabled: true
  replicas: 3
  jetstream:
    enabled: true
    fileStore:
      pvc:
        size: 10Gi

# TiKV configuration
tikv:
  enabled: true
  pd:
    replicas: 3
    storage:
      size: 10Gi
  tikv:
    replicas: 5
    storage:
      size: 50Gi

# Monitoring
monitoring:
  enabled: true
  prometheus:
    retention: 15d
    storage:
      size: 50Gi
  grafana:
    adminPassword: admin
    dashboards:
      claw-harness:
        json: |
          {
            "dashboard": {
              "title": "Claw-Harness Overview",
              "panels": []
            }
          }

# Security
security:
  podSecurityPolicy:
    enabled: false
  networkPolicy:
    enabled: true
  mTLS:
    enabled: true
    issuer: self-signed

# API Keys (use external secrets in production)
apiKeys:
  anthropic: ""
  openai: ""
  stripe: ""

# Logging
logging:
  level: info
  format: json
  elasticsearch:
    enabled: false
    host: ""
```

## values.production.yaml

```yaml
# Production overrides

claw:
  replicaCount: 5
  resources:
    limits:
      cpu: 4
      memory: 8Gi
    requests:
      cpu: 2
      memory: 4Gi
  autoscaling:
    minReplicas: 5
    maxReplicas: 20
  podDisruptionBudget:
    minAvailable: 3
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

nats:
  replicas: 5
  jetstream:
    fileStore:
      pvc:
        size: 100Gi
        storageClass: fast-ssd

tikv:
  pd:
    replicas: 5
  tikv:
    replicas: 9
    storage:
      size: 500Gi
      storageClass: fast-ssd

monitoring:
  prometheus:
    retention: 30d
    storage:
      size: 200Gi

security:
  mTLS:
    enabled: true
    issuer: letsencrypt-prod
  networkPolicy:
    enabled: true
    defaultDeny: true

ingress:
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
```

## Installation

```bash
# Add Helm repositories
helm repo add claw-harness https://claw-harness.github.io/charts
helm repo add nats https://nats-io.github.io/k8s/helm/charts/
helm repo add tikv https://tikv.github.io/charts/
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts/

# Install with defaults (development)
helm install claw-harness claw-harness/claw-harness \
  --namespace claw-system \
  --create-namespace

# Install with production values
helm install claw-harness claw-harness/claw-harness \
  --namespace claw-system \
  --create-namespace \
  -f values.production.yaml \
  --set apiKeys.anthropic=$ANTHROPIC_API_KEY \
  --set apiKeys.openai=$OPENAI_API_KEY

# Upgrade existing installation
helm upgrade claw-harness claw-harness/claw-harness \
  --namespace claw-system \
  -f values.production.yaml

# Uninstall
helm uninstall claw-harness -n claw-system
```

## Monitoring

```bash
# Check deployment status
helm status claw-harness -n claw-system

# View resources
kubectl get all -n claw-system

# Access Grafana
kubectl port-forward svc/grafana -n claw-system 3000:80

# Access Prometheus
kubectl port-forward svc/prometheus -n claw-system 9090:80
```

## Backup & Restore

```bash
# Backup TiKV data
tiup br backup full \
  --pd pd.claw-system:2379 \
  --storage "s3://bucket/backups" \
  --s3.region us-east-1

# Restore from backup
tiup br restore full \
  --pd pd.claw-system:2379 \
  --storage "s3://bucket/backups"

# Backup Helm values
helm get values claw-harness -n claw-system > backup-values.yaml
```

## Troubleshooting

```bash
# Check pod logs
kubectl logs -n claw-system claw-harness-0

# Describe pod
kubectl describe pod -n claw-system claw-harness-0

# Check events
kubectl get events -n claw-system --sort-by='.lastTimestamp'

# Debug connectivity
kubectl run debug --image=busybox -n claw-system -it --rm -- bash
```
