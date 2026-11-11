//! Server definition and MCP handler routing.

use rmcp::model::CallToolResult;
use rmcp::{ServerHandler, handler::server::wrapper::Parameters, tool, tool_handler, tool_router};

use crate::tools::anki::{self, BundleAnkiFlashcardArgs};
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

    /// Bundle a flashcard into an Anki deck package (.apkg).
    #[tool(
        name = "bundle-anki-flashcard",
        description = "Bundle a flashcard into a downloadable Anki deck package (.apkg) file with the specified deck name, question, and answer."
    )]
    pub async fn bundle_anki_flashcard(
        &self,
        Parameters(args): Parameters<BundleAnkiFlashcardArgs>,
    ) -> CallToolResult {
        tracing::debug!(
            deck = ?args.deck_name,
            question_len = args.question.len(),
            "executing bundle-anki-flashcard action"
        );
        anki::bundle_flashcard(&args)
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

    #[tokio::test]
    async fn test_bundle_anki_flashcard_tool_execution() {
        let server = SandboxServer::new();
        let result = server
            .bundle_anki_flashcard(Parameters(BundleAnkiFlashcardArgs {
                deck_name: Some("Science".to_string()),
                question: "What is H2O?".to_string(),
                answer: "Water".to_string(),
            }))
            .await;
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 2);
    }

    #[test]
    fn test_server_capabilities_enable_tools() {
        let server = SandboxServer::new();
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
    }
}
