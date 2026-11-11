# Design: Anki Flashcard Action (`bundle-anki-flashcard`)

## Objective

Allow AI agents to create flashcards for Anki in headless or cloud environments (such as Cloud Run) without requiring a running Anki desktop instance or violating AnkiWeb's Terms of Service. The tool bundles flashcards directly into a standard Anki deck package (`.apkg`) and returns it as an MCP embedded resource blob.

## Architectural Trade-offs & Decisions

1. **Why not AnkiWeb direct HTTP scraping / sync protocol?**
   - AnkiWeb (`ankiuser.net`) explicitly forbids automated third-party scripts, scrapers, and unapproved sync clients in its Terms of Service. Hitting `ankiweb.net/edit/save` directly introduces fragility (CSRF/bot challenges) and account ban risks.
2. **Why not AnkiConnect on Cloud Run?**
   - AnkiConnect requires a running Anki Desktop GUI client with an X11 display server (`xvfb`), which is heavy, stateful, and poorly suited for serverless/stateless workers.
3. **In-Memory `.apkg` Generation via `genanki-rs`**:
   - Compiles a complete, standard Anki SQLite collection database (`collection.anki2`) and packages it into a zip archive with zero filesystem dependency using in-memory byte buffers (`Cursor<Vec<u8>>`).
   - Completely stateless, fast (< 10ms execution), and minimal memory footprint.

## Tool Interface

Tool Name: `bundle-anki-flashcard`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub struct BundleAnkiFlashcardArgs {
    /// Target deck name (e.g. "Default", "Japanese::Vocabulary").
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
```

## Response Format

Returns a standard MCP `CallToolResult` containing two content blocks:
1. **Descriptive Text**: Human/model-readable confirmation with the deck name, sanitized filename, and package size.
2. **Embedded Resource Blob**:
   - `uri`: `memo://<SanitizedDeckName>.apkg`
   - `mimeType`: `application/octet-stream`
   - `blob`: Base64-encoded binary contents of the `.apkg` archive.

## Error Handling

- Trims input strings and returns a tool-level error (`CallToolResult::error`) if `question` or `answer` is empty or only whitespace.
- Gracefully handles deck naming edge-cases (sanitizes special characters for file URIs while preserving the exact deck hierarchy string inside the collection database).
