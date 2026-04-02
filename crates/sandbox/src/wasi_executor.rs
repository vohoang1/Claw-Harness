//! WASI-based WASM Execution Engine

use anyhow::{Result, Context};
use serde_json::Value;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, error};
use wasmtime::{Engine, Module, Store, Linker};
use wasmtime_wasi::{WasiCtxBuilder, WasiCtx};

/// WASM Sandbox Executor
pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    /// Create a new WASM sandbox executor
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Execute a WASM module with WASI support
    /// 
    /// # Arguments
    /// * `wasm_path` - Path to the WASM file
    /// * `args` - Command-line arguments to pass to the WASM module
    /// * `env_vars` - Environment variables to set
    /// 
    /// # Returns
    /// The JSON result from the WASM module (read from /tmp/result.json)
    pub async fn execute(
        &self,
        wasm_path: &Path,
        args: &[String],
        env_vars: &[(String, String)],
    ) -> Result<Value> {
        info!("Executing WASM module: {:?}", wasm_path);
        
        let wasm_bytes = fs::read(wasm_path).await
            .context("Failed to read WASM file")?;
        
        let module = Module::from_binary(&self.engine, &wasm_bytes)
            .context("Failed to compile WASM module")?;
        
        let mut linker = Linker::new(&self.engine);
        
        // Build WASI context
        let mut wasi_builder = WasiCtxBuilder::new()
            .inherit_stdio()
            .args(args)
            .context("Failed to set WASI args")?;
        
        // Set environment variables
        for (key, value) in env_vars {
            wasi_builder = wasi_builder.env(key, value)
                .context("Failed to set WASI env var")?;
        }
        
        // Add preopened directory for temp files
        wasi_builder = wasi_builder.preopen_dir(
            std::env::temp_dir(),
            "/tmp"
        ).context("Failed to preopen /tmp")?;
        
        let wasi = wasi_builder.build();
        
        // Link WASI to the module
        wasmtime_wasi::add_to_linker(&mut linker, |cx| cx)
            .context("Failed to link WASI")?;
        
        let mut store = Store::new(&self.engine, wasi);
        
        // Instantiate the module
        let instance = linker.instantiate_async(&mut store, &module).await
            .context("Failed to instantiate WASM module")?;
        
        // Get and call the _start function (WASI entry point)
        if let Some(start_func) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
            start_func.call_async(&mut store, ()).await
                .context("WASM execution failed")?;
        } else {
            anyhow::bail!("WASM module does not have _start function");
        }
        
        // Read result from /tmp/result.json
        let result_path = std::env::temp_dir().join("claw_wasm_result.json");
        let result_content = fs::read_to_string(&result_path).await
            .context("Failed to read WASM result")?;
        
        let result: Value = serde_json::from_str(&result_content)
            .context("Failed to parse WASM result as JSON")?;
        
        // Cleanup
        let _ = fs::remove_file(&result_path).await;
        
        info!("WASM execution completed successfully");
        Ok(result)
    }

    /// Execute a WASM module from bytes
    pub async fn execute_bytes(
        &self,
        wasm_bytes: &[u8],
        args: &[String],
        env_vars: &[(String, String)],
    ) -> Result<Value> {
        let temp_path = std::env::temp_dir().join(format!("claw_{}.wasm", uuid::Uuid::new_v4()));
        fs::write(&temp_path, wasm_bytes).await?;
        
        let result = self.execute(&temp_path, args, env_vars).await;
        
        // Cleanup
        let _ = fs::remove_file(&temp_path).await;
        
        result
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM sandbox")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let sandbox = WasmSandbox::new();
        assert!(sandbox.is_ok());
    }
}
