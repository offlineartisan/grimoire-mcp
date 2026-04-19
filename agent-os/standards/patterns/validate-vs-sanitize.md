# Validate vs Sanitize

```rust
fn validate_pattern_name(name: &str) -> Result<(), McpError> { ... }
fn sanitize_filename(name: &str) -> String {
    name.replace(' ', "-").to_lowercase()
}
```

Rules: validate first, only sanitize already-valid input. Display name preserves original; filesystem name uses sanitized form. Never silently "fix" invalid input.
