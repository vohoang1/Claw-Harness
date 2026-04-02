//! Distributed State Store using TiKV
//! 
//! Provides strongly-consistent distributed key-value storage.

use anyhow::Result;
use tikv_client::{Config, RawClient};
use tracing::{info, warn};

use crate::{Job, JobId, SessionId};

/// TiKV-based distributed state store
pub struct StateStore {
    client: RawClient,
}

impl StateStore {
    /// Create a new state store connected to TiKV
    pub async fn new(pd_endpoints: Vec<String>) -> Result<Self> {
        let config = Config::new(pd_endpoints);
        let client = RawClient::new(config).await?;
        info!("Connected to TiKV cluster");
        
        Ok(Self { client })
    }

    /// Store a job
    pub async fn put_job(&self, job: &Job) -> Result<()> {
        let key = format!("job:{}", job.id);
        let value = serde_json::to_vec(job)?;
        self.client.put(key.into_bytes(), value).await?;
        Ok(())
    }

    /// Retrieve a job by ID
    pub async fn get_job(&self, job_id: JobId) -> Result<Option<Job>> {
        let key = format!("job:{}", job_id);
        match self.client.get(key.into_bytes()).await? {
            Some(value) => {
                let job = serde_json::from_slice(&value)?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Store session state
    pub async fn put_session_state(&self, session_id: SessionId, data: &[u8]) -> Result<()> {
        let key = format!("session:{}", session_id);
        self.client.put(key.into_bytes(), data.to_vec()).await?;
        Ok(())
    }

    /// Retrieve session state
    pub async fn get_session_state(&self, session_id: SessionId) -> Result<Option<Vec<u8>>> {
        let key = format!("session:{}", session_id);
        self.client.get(key.into_bytes()).await
    }

    /// Delete a job
    pub async fn delete_job(&self, job_id: JobId) -> Result<()> {
        let key = format!("job:{}", job_id);
        self.client.delete(key.into_bytes()).await?;
        Ok(())
    }

    /// Store arbitrary key-value data
    pub async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        self.client.put(key.as_bytes().to_vec(), value.to_vec()).await?;
        Ok(())
    }

    /// Retrieve arbitrary data by key
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.client.get(key.as_bytes().to_vec()).await
    }
}

/// In-memory fallback when TiKV is not available
pub struct MemoryStore {
    cache: tokio::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            cache: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cache = self.cache.read().await;
        Ok(cache.get(key).cloned())
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
