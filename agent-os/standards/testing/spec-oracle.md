# Spec Oracle, Not Production Oracle

Tests derive expected value from the spec, not the SUT.

```rust
fn expected_sanitized_filename(name: &str) -> String {
    name.replace(' ', "-").to_lowercase()
}
```

Rules: never compute expected by calling the SUT; doc-comment the helper with a design-doc pointer; when spec changes, update helper first.
