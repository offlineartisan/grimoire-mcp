# Tool Router + Handler Wiring

Two separate impl blocks.

```rust
#[tool_router]
impl Patterns {
    pub fn new() -> Self { ... }
    #[tool(description = "...")]
    fn list_patterns(&self) -> Result<CallToolResult, McpError> { ... }
}

#[tool_handler]
impl ServerHandler for Patterns { ... }
```

Rules: `#[tool_router]` block = tool methods; `#[tool_handler] impl ServerHandler` = protocol surface; struct holds `tool_router: ToolRouter<Self>`.
