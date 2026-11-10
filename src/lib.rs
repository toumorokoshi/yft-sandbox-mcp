//! `yft-sandbox-mcp` is an extensible Model Context Protocol (MCP) server
//! built in Rust for hosting custom actions and automation tools.

pub mod auth;
pub mod http;
pub mod server;
pub mod tools;

pub use auth::{AuthToken, load_auth_token, parse_auth_token};
pub use http::{build_router, default_server_config, run_http_server};
pub use server::SandboxServer;
