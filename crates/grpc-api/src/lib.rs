//! Claw-Harness gRPC API
//! 
//! High-performance RPC interface for distributed AI agent orchestration.

pub mod generated {
    tonic::include_proto!("claw.v1");
}

pub mod service;
pub mod server;

pub use service::ClawServiceHandler;
pub use server::GrpcServer;
