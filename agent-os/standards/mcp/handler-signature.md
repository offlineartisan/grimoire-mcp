# Tool Handler Signature

Destructure the request struct inline in the parameter list.

```rust
#[tool(description = "Search patterns by query, category, framework or tag")]
fn search_patterns(
    &self,
    Parameters(PatternSearchRequest {
        query, category, framework, tag,
    }): Parameters<PatternSearchRequest>,
) -> Result<CallToolResult, McpError> { ... }
```

Rules: bind via `Parameters(FooRequest { ... })`; destructure every field used; body refers to fields as locals.
