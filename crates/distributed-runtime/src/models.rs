//! Distributed Job and Task Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Unique identifier for a job
pub type JobId = Uuid;

/// Unique identifier for a session
pub type SessionId = Uuid;

/// Represents a distributed job (conversation session)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub session_id: SessionId,
    pub prompt: String,
    pub state: JobState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Job {
    pub fn new(session_id: SessionId, prompt: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            session_id,
            prompt,
            state: JobState::Pending,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn transition_to(&mut self, new_state: JobState) {
        self.state = new_state;
        self.updated_at = Utc::now();
    }
}

/// Job state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    /// Job is waiting to be scheduled
    Pending,
    /// Job is currently being processed
    Running,
    /// Job is waiting for a tool call to complete
    AwaitToolCall(ToolCall),
    /// Job completed successfully with final response
    Completed(String),
    /// Job failed with error message
    Failed(String),
}

/// Represents a tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
    pub call_id: String,
}

/// Represents a tool task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTask {
    pub id: Uuid,
    pub job_id: JobId,
    pub session_id: SessionId,
    pub tool_name: String,
    pub args: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl ToolTask {
    pub fn new(job_id: JobId, session_id: SessionId, tool_call: ToolCall) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id,
            session_id,
            tool_name: tool_call.name,
            args: tool_call.args,
            created_at: Utc::now(),
        }
    }
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub task_id: Uuid,
    pub job_id: JobId,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Node information for distributed scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub load: f64,
    pub last_heartbeat: DateTime<Utc>,
}

impl NodeInfo {
    pub fn new(id: String, host: String, port: u16) -> Self {
        Self {
            id,
            host,
            port,
            load: 0.0,
            last_heartbeat: Utc::now(),
        }
    }
}
