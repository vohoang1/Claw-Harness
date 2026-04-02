//! Tool Package Definition

use serde::{Deserialize, Serialize};

/// Tool package manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPackage {
    /// Package name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Description
    pub description: String,
    /// Author
    pub author: Option<String>,
    /// License (MIT, Apache-2.0, etc.)
    pub license: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// WASM file path in package
    pub wasm_file: String,
    /// Input schema
    pub inputs: Vec<InputSpec>,
    /// Output schema
    pub outputs: Vec<OutputSpec>,
    /// Runtime type
    pub runtime: RuntimeType,
    /// SHA256 checksum
    pub checksum: String,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Input specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
}

/// Output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub output_type: String,
    pub description: Option<String>,
}

/// Runtime type for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Wasm,
    Firecracker,
    Native,
}

/// Tool package archive format (.clawpkg)
pub struct ClawPackage {
    pub manifest: ToolPackage,
    pub wasm_bytes: Vec<u8>,
}

impl ClawPackage {
    /// Create a new package from manifest and WASM bytes
    pub fn new(manifest: ToolPackage, wasm_bytes: Vec<u8>) -> Self {
        Self { manifest, wasm_bytes }
    }

    /// Serialize package to tarball
    pub fn to_tar(&self) -> anyhow::Result<Vec<u8>> {
        use std::io::Cursor;
        use tar::Builder;

        let mut tar_builder = Builder::new(Vec::new());
        
        // Add manifest.yaml
        let manifest_yaml = serde_yaml::to_string(&self.manifest)?;
        tar_builder.append_file(
            "manifest.yaml",
            &mut Cursor::new(manifest_yaml.as_bytes()),
        )?;
        
        // Add WASM file
        tar_builder.append_file(
            &self.manifest.wasm_file,
            &mut Cursor::new(&self.wasm_bytes),
        )?;
        
        Ok(tar_builder.into_inner()?)
    }

    /// Deserialize package from tarball
    pub fn from_tar(data: &[u8]) -> anyhow::Result<Self> {
        use std::io::Cursor;
        use tar::Archive;

        let mut archive = Archive::new(Cursor::new(data));
        let mut manifest_bytes = None;
        let mut wasm_bytes = None;

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_string_lossy().to_string();
            
            if path == "manifest.yaml" {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents)?;
                manifest_bytes = Some(contents);
            } else {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents)?;
                wasm_bytes = Some(contents);
            }
        }

        let manifest: ToolPackage = serde_yaml::from_slice(
            &manifest_bytes.ok_or_else(|| anyhow::anyhow!("manifest.yaml not found"))?
        )?;

        Ok(Self::new(
            manifest,
            wasm_bytes.ok_or_else(|| anyhow::anyhow!("WASM file not found"))?,
        ))
    }
}
