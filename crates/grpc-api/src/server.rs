//! gRPC Server

use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, error};

use crate::service::ClawServiceHandler;

/// gRPC Server Configuration
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    pub host: String,
    pub port: u16,
    pub max_message_size: usize,
    pub concurrency_limit: Option<usize>,
    pub timeout: Option<std::time::Duration>,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            max_message_size: 10 * 1024 * 1024, // 10MB
            concurrency_limit: Some(100),
            timeout: Some(std::time::Duration::from_secs(30)),
        }
    }
}

/// gRPC Server
pub struct GrpcServer {
    config: GrpcServerConfig,
    handler: ClawServiceHandler,
}

impl GrpcServer {
    pub fn new(config: GrpcServerConfig) -> Self {
        Self {
            handler: ClawServiceHandler::new(),
            config,
        }
    }

    /// Run the gRPC server
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()?;

        info!("Starting gRPC server on {}", addr);

        let mut server_builder = Server::builder()
            .max_message_size(self.config.max_message_size);

        if let Some(limit) = self.config.concurrency_limit {
            server_builder = server_builder.concurrency_limit(limit as u64);
        }

        if let Some(timeout) = self.config.timeout {
            server_builder = server_builder.timeout(timeout);
        }

        let service = crate::generated::claw_service_server::ClawServiceServer::new(
            self.handler.clone(),
        );

        server_builder
            .add_service(service)
            .serve(addr)
            .await?;

        Ok(())
    }

    /// Run with TLS
    pub async fn run_with_tls(
        &self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tonic::transport::{Identity, ServerTlsConfig};
        use std::fs;

        let cert = fs::read(cert_path)?;
        let key = fs::read(key_path)?;
        let identity = Identity::from_pem(cert, key);

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()?;

        info!("Starting gRPC server with TLS on {}", addr);

        let tls_config = ServerTlsConfig::new().identity(identity);

        Server::builder()
            .tls_config(tls_config)?
            .add_service(crate::generated::claw_service_server::ClawServiceServer::new(
                self.handler.clone(),
            ))
            .serve(addr)
            .await?;

        Ok(())
    }
}
