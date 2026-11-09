//! `yft-sandbox-mcp` is an extensible Model Context Protocol (MCP) server
//! built in Rust for hosting custom actions and automation tools.

pub mod http;
pub mod server;
pub mod tools;

pub use http::{build_router, default_server_config, run_http_server};
pub use server::SandboxServer;
