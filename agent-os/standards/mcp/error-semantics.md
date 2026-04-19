# MCP Error Semantics

Three-way split. Pick by who can act on the result.

| Return                          | When                                                  |
|---------------------------------|-------------------------------------------------------|
| `CallToolResult::success(text)` | Tool ran correctly — result may be empty or negative. |
| `McpError::invalid_params`      | Caller's input breaches the contract.                 |
| `McpError::internal_error`      | Execution failure (IO, env, permission).              |

```rust
None => Ok(CallToolResult::success(vec![
    Content::text(format!("Pattern '{}' not found.", pattern_name))
])),

return Err(McpError::invalid_params(
    "Pattern must be 1-100 characters", None));

Err(e) => Err(McpError::internal_error(
    format!("Failed to create pattern: {}", e), None)),
```

Rules:
- "Not found", empty search, zero results → `success` with text.
- Bad input → `invalid_params`. Agent fixes its call.
- Filesystem/env failure → `internal_error`. Human intervention.
