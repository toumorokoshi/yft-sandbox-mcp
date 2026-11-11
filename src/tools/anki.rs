//! Anki flashcard bundling action creating standard `.apkg` packages.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Cursor;

use base64::prelude::*;
use genanki_rs::{Deck, Note, Package, basic_model};
use rmcp::model::{CallToolResult, ContentBlock, ResourceContents};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default deck name used when none is specified.
pub const DEFAULT_DECK_NAME: &str = "Default";

/// MIME type for `.apkg` Anki package archives.
pub const APKG_MIME_TYPE: &str = "application/octet-stream";

/// Arguments for the `bundle-anki-flashcard` MCP action.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub struct BundleAnkiFlashcardArgs {
    /// Target deck name (e.g. "Default", "Computer Science", etc.).
    /// Defaults to "Default" if omitted or empty.
    #[serde(default)]
    pub deck_name: Option<String>,

    /// The question / front side of the flashcard.
    #[serde(alias = "front")]
    pub question: String,

    /// The answer / back side of the flashcard.
    #[serde(alias = "back")]
    pub answer: String,
}

/// Errors that can occur during flashcard bundling.
#[derive(Debug, PartialEq, Eq)]
pub enum AnkiBundleError {
    /// The question field was empty or only whitespace.
    EmptyQuestion,

    /// The answer field was empty or only whitespace.
    EmptyAnswer,

    /// An internal error occurred while generating the package.
    GenerationFailed(String),
}

impl std::fmt::Display for AnkiBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyQuestion => write!(f, "question cannot be empty"),
            Self::EmptyAnswer => write!(f, "answer cannot be empty"),
            Self::GenerationFailed(msg) => write!(f, "failed to generate .apkg package: {msg}"),
        }
    }
}

impl std::error::Error for AnkiBundleError {}

/// Deterministically derive a positive non-zero 64-bit deck ID from a deck name.
#[must_use]
pub fn calculate_deck_id(deck_name: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    deck_name.hash(&mut hasher);
    let hash = hasher.finish();
    ((hash & 0x7FFF_FFFF_FFFF_FFFF) as i64).max(1)
}

/// Sanitize a deck name for use in a file URI or download filename.
#[must_use]
pub fn sanitize_filename(deck_name: &str) -> String {
    let sanitized: String = deck_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "flashcard_deck".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Pure functional creation of an Anki `.apkg` package from a question and answer.
///
/// Returns the raw binary bytes of the compressed `.apkg` archive.
pub fn create_apkg_bundle(
    deck_name: Option<&str>,
    question: &str,
    answer: &str,
) -> Result<Vec<u8>, AnkiBundleError> {
    let question_trimmed = question.trim();
    if question_trimmed.is_empty() {
        return Err(AnkiBundleError::EmptyQuestion);
    }

    let answer_trimmed = answer.trim();
    if answer_trimmed.is_empty() {
        return Err(AnkiBundleError::EmptyAnswer);
    }

    let resolved_deck = deck_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_DECK_NAME);

    let deck_id = calculate_deck_id(resolved_deck);
    let mut deck = Deck::new(deck_id, resolved_deck, "");

    let model = basic_model();
    let note = Note::new(model, vec![question_trimmed, answer_trimmed])
        .map_err(|e| AnkiBundleError::GenerationFailed(e.to_string()))?;

    deck.add_note(note);

    let mut package = Package::new(vec![deck], vec![])
        .map_err(|e| AnkiBundleError::GenerationFailed(e.to_string()))?;

    let mut cursor = Cursor::new(Vec::new());
    package
        .write(&mut cursor)
        .map_err(|e| AnkiBundleError::GenerationFailed(e.to_string()))?;

    Ok(cursor.into_inner())
}

/// Execute the `bundle-anki-flashcard` tool and produce an MCP [`CallToolResult`].
#[must_use]
pub fn bundle_flashcard(args: &BundleAnkiFlashcardArgs) -> CallToolResult {
    let resolved_deck = args
        .deck_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_DECK_NAME);

    match create_apkg_bundle(Some(resolved_deck), &args.question, &args.answer) {
        Ok(apkg_bytes) => {
            let slug = sanitize_filename(resolved_deck);
            let filename = format!("{slug}.apkg");
            let encoded_blob = BASE64_STANDARD.encode(&apkg_bytes);

            let resource = ResourceContents::blob(encoded_blob, format!("memo://{filename}"))
                .with_mime_type(APKG_MIME_TYPE);

            CallToolResult::success(vec![
                ContentBlock::text(format!(
                    "Successfully created Anki package '{filename}' for deck '{resolved_deck}' (size: {} bytes).",
                    apkg_bytes.len()
                )),
                ContentBlock::resource(resource),
            ])
        }
        Err(err) => CallToolResult::error(vec![ContentBlock::text(format!(
            "Failed to bundle Anki flashcard: {err}"
        ))]),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use zip::ZipArchive;

    use super::*;

    #[test]
    fn test_calculate_deck_id_consistency() {
        let id1 = calculate_deck_id("Japanese::Vocabulary");
        let id2 = calculate_deck_id("Japanese::Vocabulary");
        let id3 = calculate_deck_id("Computer Science");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert!(id1 > 0);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("Japanese::Vocabulary"),
            "Japanese__Vocabulary"
        );
        assert_eq!(sanitize_filename("  "), "flashcard_deck");
        assert_eq!(sanitize_filename("Default"), "Default");
        assert_eq!(
            sanitize_filename("Math / Algebra & Geometry"),
            "Math___Algebra___Geometry"
        );
    }

    #[test]
    fn test_create_apkg_bundle_valid() {
        let bundle = create_apkg_bundle(
            Some("CS::Algorithms"),
            "What is quicksort?",
            "A divide-and-conquer sorting algorithm.",
        )
        .expect("bundle generation should succeed");

        assert!(!bundle.is_empty());
        // Verify standard ZIP header (PK\x03\x04)
        assert_eq!(&bundle[0..4], &[0x50, 0x4B, 0x03, 0x04]);

        // Verify valid zip contents containing collection.anki2
        let cursor = Cursor::new(bundle);
        let mut archive = ZipArchive::new(cursor).expect("valid zip archive");
        let mut file_names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        file_names.sort();

        assert!(file_names.contains(&"collection.anki2".to_string()));
        assert!(file_names.contains(&"media".to_string()));

        // Ensure collection.anki2 is non-empty
        let mut collection_file = archive.by_name("collection.anki2").unwrap();
        let mut collection_bytes = Vec::new();
        collection_file.read_to_end(&mut collection_bytes).unwrap();
        assert!(!collection_bytes.is_empty());
    }

    #[test]
    fn test_create_apkg_bundle_default_deck() {
        let bundle = create_apkg_bundle(None, "Question?", "Answer.")
            .expect("should default deck name to 'Default'");
        assert!(!bundle.is_empty());
    }

    #[test]
    fn test_create_apkg_bundle_empty_fields_fail() {
        assert_eq!(
            create_apkg_bundle(Some("Deck"), "   ", "Answer"),
            Err(AnkiBundleError::EmptyQuestion)
        );
        assert_eq!(
            create_apkg_bundle(Some("Deck"), "Question", "   "),
            Err(AnkiBundleError::EmptyAnswer)
        );
    }

    #[test]
    fn test_bundle_flashcard_success_result() {
        let args = BundleAnkiFlashcardArgs {
            deck_name: Some("Rust::Lang".to_string()),
            question: "What is an enum in Rust?".to_string(),
            answer: "A type that represents one of several possible variants.".to_string(),
        };

        let result = bundle_flashcard(&args);
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 2);

        // First block: descriptive text
        let text_block = result.content[0].as_text().expect("text block");
        assert!(text_block.text.contains("Rust__Lang.apkg"));

        // Second block: embedded resource blob
        let resource_block = result.content[1].as_resource().expect("resource block");
        match &resource_block.resource {
            ResourceContents::BlobResourceContents {
                uri,
                mime_type,
                blob,
                ..
            } => {
                assert_eq!(uri, "memo://Rust__Lang.apkg");
                assert_eq!(mime_type.as_deref(), Some(APKG_MIME_TYPE));
                let decoded = BASE64_STANDARD.decode(blob).expect("valid base64");
                assert_eq!(&decoded[0..4], &[0x50, 0x4B, 0x03, 0x04]);
            }
            _ => panic!("expected blob resource"),
        }
    }

    #[test]
    fn test_bundle_flashcard_error_result() {
        let args = BundleAnkiFlashcardArgs {
            deck_name: None,
            question: "".to_string(),
            answer: "Some answer".to_string(),
        };

        let result = bundle_flashcard(&args);
        assert_eq!(result.is_error, Some(true));
        let text_block = result.content[0].as_text().expect("text block");
        assert!(text_block.text.contains("question cannot be empty"));
    }
}
