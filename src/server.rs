//! Server definition and MCP handler routing.

use rmcp::{ServerHandler, handler::server::wrapper::Parameters, tool, tool_handler, tool_router};

use crate::tools::echo::{self, EchoArgs};

/// Sandbox MCP server exposing custom tools and actions.
#[derive(Debug, Clone, Default)]
pub struct SandboxServer;

impl SandboxServer {
    /// Create a new server instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[tool_router]
impl SandboxServer {
    /// Example MCP action that echoes back the input message.
    #[tool(
        name = "echo",
        description = "Example MCP action that echoes back the input message."
    )]
    pub async fn echo(&self, Parameters(args): Parameters<EchoArgs>) -> String {
        tracing::debug!(message = %args.message, "executing echo action");
        echo::format_echo(&args.message)
    }
}

#[tool_handler]
impl ServerHandler for SandboxServer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_tool_execution() {
        let server = SandboxServer::new();
        let result = server
            .echo(Parameters(EchoArgs {
                message: "testing".to_string(),
            }))
            .await;
        assert_eq!(result, "Echo: testing");
    }

    #[test]
    fn test_server_capabilities_enable_tools() {
        let server = SandboxServer::new();
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
    }
}
