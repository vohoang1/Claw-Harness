//! Claw-Harness Metrics & Observability
//! 
//! Provides Prometheus metrics and OpenTelemetry tracing support.

pub mod prometheus_metrics;
pub mod tracing_setup;

pub use prometheus_metrics::*;
pub use tracing_setup::*;
