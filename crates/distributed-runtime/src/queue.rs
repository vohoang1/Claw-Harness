//! Distributed Task Queue using NATS
//! 
//! Provides publish/subscribe functionality for tool tasks.

use anyhow::Result;
use nats::asynk::Connection;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{info, warn, error};

use crate::ToolTask;

/// NATS-based distributed task queue
pub struct TaskQueue {
    conn: Connection,
    subject: String,
}

impl TaskQueue {
    /// Create a new task queue connected to NATS
    pub async fn new(nats_url: &str) -> Result<Self> {
        let conn = nats::asynk::connect(nats_url)?;
        info!("Connected to NATS at {}", nats_url);
        
        Ok(Self {
            conn,
            subject: "claw.tasks".to_string(),
        })
    }

    /// Publish a task to the queue
    pub async fn publish(&self, task: &ToolTask) -> Result<()> {
        let payload = serde_json::to_vec(task)?;
        self.conn.publish(&self.subject, payload)?;
        info!("Published task {} to queue", task.id);
        Ok(())
    }

    /// Subscribe to tasks from the queue
    pub async fn subscribe(&self) -> Result<impl futures::stream::Stream<Item = ToolTask>> {
        let sub = self.conn.subscribe(&self.subject)?;
        Ok(sub.filter_map(|msg| async move {
            match serde_json::from_slice(&msg.data) {
                Ok(task) => Some(task),
                Err(e) => {
                    warn!("Failed to deserialize task: {}", e);
                    None
                }
            }
        }))
    }

    /// Publish task result
    pub async fn publish_result<T: Serialize>(&self, result: &T) -> Result<()> {
        let payload = serde_json::to_vec(result)?;
        self.conn.publish("claw.results", payload)?;
        Ok(())
    }

    /// Subscribe to task results
    pub async fn subscribe_results<T: DeserializeOwned + Send + 'static>(
        &self,
    ) -> Result<impl futures::stream::Stream<Item = T>> {
        let sub = self.conn.subscribe("claw.results")?;
        Ok(sub.filter_map(|msg| async move {
            match serde_json::from_slice(&msg.data) {
                Ok(result) => Some(result),
                Err(e) => {
                    warn!("Failed to deserialize result: {}", e);
                    None
                }
            }
        }))
    }
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        if let Err(e) = self.conn.flush() {
            error!("Failed to flush NATS connection on drop: {}", e);
        }
    }
}
