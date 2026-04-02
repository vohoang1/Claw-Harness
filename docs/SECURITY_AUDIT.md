# Security Audit & Penetration Testing Guide

Comprehensive security testing for Claw-Harness platform.

## Security Architecture

### Defense in Depth

```
┌─────────────────────────────────────────┐
│  Perimeter Security (WAF, DDoS)         │
├─────────────────────────────────────────┤
│  Authentication (OAuth, API Keys)       │
├─────────────────────────────────────────┤
│  Authorization (OPA Policies)           │
├─────────────────────────────────────────┤
│  WASM Sandbox Isolation                 │
├─────────────────────────────────────────┤
│  Network Policies (mTLS, NetworkPolicy) │
├─────────────────────────────────────────┤
│  Data Encryption (at rest, in transit)  │
└─────────────────────────────────────────┘
```

## Automated Security Scanning

### 1. Dependency Audit

```bash
# Rust dependencies
cargo audit

# Python dependencies
pip-audit
safety check

# Container scanning
trivy image claw-harness:latest
grype claw-harness:latest
```

### 2. Static Analysis

```bash
# Rust
cargo clippy -- -D warnings
cargo deny check

# Python
bandit -r python/
ruff check python/
```

### 3. Secret Detection

```bash
# Detect hardcoded secrets
gitleaks detect --source . --verbose

# Pre-commit hook
pre-commit run --all-files
```

## Penetration Testing

### 1. API Security Testing

#### OWASP API Top 10

```bash
# Install OWASP ZAP
docker run -t owasp/zap2docker-stable zap-baseline.py \
    -t http://localhost:8080/api/v1/

# Test authentication bypass
curl -X POST http://localhost:8080/api/v1/run \
    -H "Authorization: Bearer invalid_token" \
    -d '{"prompt": "test"}'

# Test injection
curl -X POST http://localhost:8080/api/v1/run \
    -H "Content-Type: application/json" \
    -d '{"prompt": "{{constructor.constructor(\"return this.process\")()}}"}'

# Test rate limiting
for i in {1..1000}; do
    curl -s http://localhost:8080/health > /dev/null &
done
```

### 2. WASM Sandbox Escape Tests

```rust
// tests/security/test_wasm_escape.rs

#[tokio::test]
async fn test_filesystem_access() {
    // Attempt to read /etc/passwd
    let wasm = include_bytes!("fixtures/malicious_fs.wasm");
    let result = sandbox.execute_bytes(wasm, &[], &[]).await;
    
    // Should fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_network_access() {
    // Attempt to make network call
    let wasm = include_bytes!("fixtures/malicious_network.wasm");
    let result = sandbox.execute_bytes(wasm, &[], &[]).await;
    
    // Should fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_environment_variables() {
    // Attempt to read sensitive env vars
    let wasm = include_bytes!("fixtures/malicious_env.wasm");
    let result = sandbox.execute_bytes(wasm, &[], &[]).await;
    
    // Should only see allowed env vars
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output["secrets"].is_string());
}
```

### 3. Distributed System Tests

```bash
# Test TiKV access control
tikv-ctl --host localhost:20160 security dump-keys

# Test NATS authentication
nats sub '>' --server nats://localhost:4222

# Test inter-node communication
# Attempt to inject malicious node
docker run --name malicious-node claw-harness:latest \
    --node-id malicious \
    --inject-bad-data
```

### 4. Kubernetes Security

```bash
# Check pod security policies
kubectl get psp

# Test network policies
kubectl run test-pod --image=busybox --rm -it -- bash
# Try to reach claw-harness pods (should be blocked)

# Check RBAC
kubectl auth can-i --list

# Test privilege escalation
kubectl run privileged --image=busybox \
    --overrides='{"spec":{"containers":[{"name":"privileged","image":"busybox","securityContext":{"privileged":true}}]}}'
```

## Security Checklist

### Authentication & Authorization

- [ ] OAuth 2.0 / OIDC implemented correctly
- [ ] API keys rotated regularly
- [ ] JWT tokens validated (signature, expiry, issuer)
- [ ] RBAC policies enforced
- [ ] OPA policies tested and audited

### Data Protection

- [ ] TLS 1.3 for all communications
- [ ] Secrets encrypted at rest
- [ ] Database connections encrypted
- [ ] No sensitive data in logs
- [ ] PII handling compliant with GDPR

### Container Security

- [ ] Running as non-root user
- [ ] Read-only root filesystem
- [ ] No unnecessary capabilities
- [ ] Resource limits set
- [ ] Image scanning enabled

### Network Security

- [ ] Network policies enforced
- [ ] mTLS between services
- [ ] Ingress rate limiting
- [ ] DDoS protection enabled
- [ ] Firewall rules configured

### Monitoring & Incident Response

- [ ] Security events logged
- [ ] Anomaly detection enabled
- [ ] Alert thresholds configured
- [ ] Incident response plan documented
- [ ] Regular security drills conducted

## Vulnerability Response

### Severity Levels

| Level | Response Time | Example |
|-------|--------------|---------|
| Critical | < 4 hours | RCE, auth bypass |
| High | < 24 hours | SQL injection, XSS |
| Medium | < 1 week | Information disclosure |
| Low | < 1 month | Minor configuration issues |

### Response Process

1. **Detection** - Automated scan or manual report
2. **Triage** - Assess severity and impact
3. **Containment** - Immediate mitigation
4. **Eradication** - Fix the root cause
5. **Recovery** - Deploy fix and verify
6. **Lessons Learned** - Document and improve

## Security Tools

### Scanning Tools

```yaml
# cargo-audit - Rust dependency vulnerabilities
# trivy - Container vulnerability scanner
# gitleaks - Secret detection
# bandit - Python security linting
# OWASP ZAP - API security testing
# nmap - Network scanning
# burpsuite - Web application testing
```

### Monitoring Tools

```yaml
# Falco - Runtime security monitoring
# osquery - Host visibility
# Wazuh - SIEM
# Prometheus + Grafana - Metrics monitoring
# Jaeger - Distributed tracing
```

## Compliance

### SOC 2 Type II

- Access controls documented
- Change management procedures
- Incident response plan
- Regular security assessments

### GDPR

- Data minimization
- Right to erasure
- Data portability
- Privacy by design

### ISO 27001

- ISMS implemented
- Risk assessment conducted
- Controls implemented
- Regular audits

## Security Contacts

- Security Team: security@claw-harness.io
- Bug Bounty: https://hackerone.com/claw-harness
- PGP Key: Available on keybase.io

## Reporting Vulnerabilities

Please report security vulnerabilities to security@claw-harness.io

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (if any)

We respond within 48 hours for all security reports.
