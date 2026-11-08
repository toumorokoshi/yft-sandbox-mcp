//! `yft-sandbox-mcp` is an extensible Model Context Protocol (MCP) server
//! built in Rust for hosting custom actions and automation tools.

pub mod server;
pub mod tools;

pub use server::SandboxServer;
