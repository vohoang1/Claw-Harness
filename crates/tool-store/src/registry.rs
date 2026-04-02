//! Tool Registry - Marketplace for plugins

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::ToolPackage;

/// Registry index (list of available tools)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryIndex {
    pub tools: Vec<ToolEntry>,
    pub last_updated: String,
}

/// Tool entry in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub downloads: u64,
    pub download_url: String,
    pub checksum: String,
}

/// Tool registry client
pub struct ToolRegistry {
    registry_url: String,
    cache_dir: PathBuf,
}

impl ToolRegistry {
    /// Create a new registry client
    pub fn new(registry_url: &str, cache_dir: PathBuf) -> Self {
        Self {
            registry_url: registry_url.to_string(),
            cache_dir,
        }
    }

    /// Fetch registry index
    pub async fn fetch_index(&self) -> Result<RegistryIndex> {
        let client = reqwest::Client::new();
        let url = format!("{}/index.json", self.registry_url);
        
        let response = client.get(&url).send().await?;
        let index: RegistryIndex = response.json().await?;
        
        info!("Fetched registry index with {} tools", index.tools.len());
        Ok(index)
    }

    /// Download a tool package
    pub async fn download_tool(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let client = reqwest::Client::new();
        let url = format!("{}/packages/{}-{}/{}.clawpkg", 
            self.registry_url, name, version, name);
        
        info!("Downloading tool {} v{} from {}", name, version, url);
        
        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?.to_vec();
        
        // Save to cache
        let cache_path = self.cache_dir.join(format!("{}-{}.clawpkg", name, version));
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&cache_path, &bytes).await?;
        
        Ok(bytes)
    }

    /// Search for tools
    pub async fn search(&self, query: &str) -> Result<Vec<ToolEntry>> {
        let index = self.fetch_index().await?;
        
        let query_lower = query.to_lowercase();
        let results = index.tools
            .into_iter()
            .filter(|tool| {
                tool.name.to_lowercase().contains(&query_lower) ||
                tool.description.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        Ok(results)
    }

    /// Get installed tools
    pub fn list_installed(&self) -> Result<Vec<ToolPackage>> {
        let mut installed = Vec::new();
        
        if !self.cache_dir.exists() {
            return Ok(installed);
        }

        let mut entries = tokio::task::block_in_place(|| {
            std::fs::read_dir(&self.cache_dir)
        })?;

        while let Some(entry) = entries.next_entry()? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "clawpkg") {
                // Read and parse manifest from package
                if let Ok(package) = self.read_package_manifest(&path) {
                    installed.push(package);
                }
            }
        }

        Ok(installed)
    }

    /// Read package manifest
    fn read_package_manifest(&self, path: &PathBuf) -> Result<ToolPackage> {
        let bytes = std::fs::read(path)?;
        let package = crate::ClawPackage::from_tar(&bytes)?;
        Ok(package.manifest)
    }

    /// Install a tool from registry
    pub async fn install(&self, name: &str, version: Option<&str>) -> Result<()> {
        let index = self.fetch_index().await?;
        
        let tool_entry = index.tools
            .iter()
            .find(|t| t.name == name && version.map_or(true, |v| t.version == v))
            .ok_or_else(|| anyhow::anyhow!("Tool {} not found", name))?;
        
        let version = version.unwrap_or(&tool_entry.version);
        
        // Download
        let bytes = self.download_tool(name, version).await?;
        
        // Verify checksum
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(&bytes);
        let checksum = hex::encode(hash);
        
        if checksum != tool_entry.checksum {
            anyhow::bail!("Checksum mismatch! Expected {}, got {}", tool_entry.checksum, checksum);
        }
        
        info!("Successfully installed {} v{}", name, version);
        Ok(())
    }

    /// Uninstall a tool
    pub fn uninstall(&self, name: &str) -> Result<()> {
        let mut entries = std::fs::read_dir(&self.cache_dir)?;
        
        while let Some(entry) = entries.next_entry()? {
            let path = entry.path();
            if path.file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.starts_with(name))
            {
                std::fs::remove_file(&path)?;
                info!("Uninstalled {}", name);
                return Ok(());
            }
        }
        
        anyhow::bail!("Tool {} not found", name);
    }
}
