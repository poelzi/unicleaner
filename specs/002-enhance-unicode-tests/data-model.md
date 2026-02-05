# Data Model for Enhance Unicode Test Suite

This feature primarily enhances testing and does not introduce new data entities. Existing entities from the core unicleaner project (e.g., Violation, ScanResult) will be used in test assertions.

## Key Entities (Reused)

- **Violation**: Represents a detected Unicode issue.
  - Fields: location (file path, line), type (e.g., bidi_override, homoglyph), description.
  - Relationships: Associated with ScanResult.
  - Validation: Must have non-empty description and valid location.

No new state transitions or entities required.
