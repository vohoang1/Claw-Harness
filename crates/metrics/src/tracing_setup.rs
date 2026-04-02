//! OpenTelemetry Tracing Setup

use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace::{self, Sampler}, Resource};
use tracing_subscriber::{layer::SubscriberExt, Registry};

/// Initialize OpenTelemetry tracing
/// 
/// # Arguments
/// * `service_name` - Name of the service (e.g., "claw-harness-node-1")
/// * `endpoint` - OTLP endpoint (e.g., "http://localhost:4317")
pub fn init_tracing(service_name: &str, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

/// Shutdown tracing and cleanup resources
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

/// Create a span for tool execution
#[macro_export]
macro_rules! tool_span {
    ($tool_name:expr, $tool_args:expr) => {
        tracing::info_span!(
            "tool_execution",
            tool_name = $tool_name,
            tool_args = %serde_json::to_string($tool_args).unwrap_or_default()
        )
    };
}
