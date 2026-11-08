# Design: Anki Flashcard Action

## Objective

Allow AI agents to create flashcards in Anki directly via an MCP tool call (`anki_create_flashcard`).

## Integration Mechanism

- **Target Service**: [AnkiConnect](https://foosoft.net/projects/anki-connect/) running locally on `http://127.0.0.1:8765`.
- **API Action**: `addNote`.
- **Card Format**:
  - `deckName`: Target deck (e.g., "Default" or user-configured).
  - `modelName`: "Basic" or "Cloze".
  - `fields`: Front/Back (or Text for Cloze).
  - `tags`: Optional list of string tags for card categorization.

## Tool Interface

```rust
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateFlashcardArgs {
    pub deck_name: Option<String>,
    pub front: String,
    pub back: String,
    pub tags: Option<Vec<String>>,
}
```

## Error Handling

- Return meaningful MCP error messages if Anki is not running or AnkiConnect plugin is not installed/accessible on port 8765.
- Validate non-empty fields before dispatching HTTP request to AnkiConnect.
