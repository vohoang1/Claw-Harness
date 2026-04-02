//! Prometheus Metrics for Claw-Harness

use lazy_static::lazy_static;
use prometheus::{IntCounter, IntGauge, Histogram, register_int_counter, register_int_gauge, register_histogram};

lazy_static! {
    /// Total number of sessions created
    pub static ref SESSIONS_TOTAL: IntCounter =
        register_int_counter!(opts!("claw_sessions_total", "Total number of sessions created")).unwrap();
    
    /// Current number of running tasks
    pub static ref TASKS_RUNNING: IntGauge =
        register_int_gauge!(opts!("claw_tasks_running", "Current number of running tasks")).unwrap();
    
    /// Total number of tool calls
    pub static ref TOOL_CALLS_TOTAL: IntCounter =
        register_int_counter!(opts!("claw_tool_calls_total", "Total number of tool calls")).unwrap();
    
    /// Tool call latency histogram
    pub static ref TOOL_LATENCY: Histogram = register_histogram!(
        "claw_tool_latency_seconds",
        "Latency of tool calls in seconds",
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    ).unwrap();
    
    /// Cache hits counter
    pub static ref CACHE_HITS_TOTAL: IntCounter =
        register_int_counter!(opts!("claw_cache_hits_total", "Total number of cache hits")).unwrap();
    
    /// Cache misses counter
    pub static ref CACHE_MISSES_TOTAL: IntCounter =
        register_int_counter!(opts!("claw_cache_misses_total", "Total number of cache misses")).unwrap();
    
    /// LLM token usage counter
    pub static ref LLM_TOKENS_TOTAL: IntCounter =
        register_int_counter!(opts!("claw_llm_tokens_total", "Total number of LLM tokens used")).unwrap();
    
    /// Error counter by tool name
    pub static ref TOOL_ERRORS_TOTAL: IntCounter =
        register_int_counter!(opts!("claw_tool_errors_total", "Total number of tool errors")).unwrap();
}

/// Initialize metrics (call once at startup)
pub fn init_metrics() -> Result<(), prometheus::Error> {
    // Force lazy_static initialization
    let _ = SESSIONS_TOTAL.get();
    let _ = TASKS_RUNNING.get();
    let _ = TOOL_CALLS_TOTAL.get();
    let _ = TOOL_LATENCY.get();
    let _ = CACHE_HITS_TOTAL.get();
    let _ = CACHE_MISSES_TOTAL.get();
    let _ = LLM_TOKENS_TOTAL.get();
    let _ = TOOL_ERRORS_TOTAL.get();
    Ok(())
}

/// Get Prometheus metrics endpoint content
pub fn gather_metrics() -> String {
    prometheus::gather().iter()
        .map(|m| prometheus::proto::MetricFamily::write_delimited(m))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default()
        .into_iter()
        .collect::<String>()
}
