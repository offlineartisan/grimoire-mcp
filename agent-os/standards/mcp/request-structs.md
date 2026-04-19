# MCP Tool Request Structs

Every tool with arguments gets a dedicated request struct.

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPatternRequest {
    #[schemars(description = "Pattern Name")]
    pattern_name: String,
}
```

Rules: name `<Verb><Noun>Request`; derive `Debug, Deserialize, schemars::JsonSchema`; every field has `#[schemars(description)]`; optional fields `Option<T>` with `#[serde(default)]` where appropriate.
