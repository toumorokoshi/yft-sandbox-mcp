use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Arguments for the example echo action.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub struct EchoArgs {
    /// The message to echo back.
    pub message: String,
}

/// Pure functional transformation for the echo action.
#[must_use]
pub fn format_echo(message: &str) -> String {
    format!("Echo: {message}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_echo_non_empty() {
        assert_eq!(format_echo("hello world"), "Echo: hello world");
    }

    #[test]
    fn test_format_echo_empty() {
        assert_eq!(format_echo(""), "Echo: ");
    }
}
