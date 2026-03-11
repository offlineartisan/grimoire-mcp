# Grimoire-MCP - Development Pattern Library

As I am leveling up my development skills, I always needed a way to quickly store some best practices I ended up implementing in projects. What if my coding assistant could just save it to some ... ✨ pattern library ✨. Well now it can! Oh, and it can read code patterns too! 🥳

So, the Grimoire-MCP is a Model Context Protocol (MCP) server for managing software development patterns stored as markdown files with YAML frontmatter. Oh, and it's all on your filesystem!

## Setup

### Required Environment Variable

The server **requires** the `PATTERNS_DIR` environment variable to be set ideally via your MCP config, or in the shell:

```bash
export PATTERNS_DIR="/path/to/your/patterns"
```

The server will fail to start if this variable is not set or if the directory doesn't exist.

### Pattern File Format

The LLM will create patterns on your behalf, but you are more than welcome to store them yourself as well.

Patterns are stored as markdown files with YAML frontmatter:

```markdown
---
pattern: example-pattern
category: rust
framework: axum
projects: [project1, project2] # These are the names of the projects this pattern was involved in.
tags: [web, api, error-handling]
---

Your pattern content goes here...
```

## Usage


### Available Tools

- `list_patterns` - List all available patterns
- `search_patterns` - Search by query, category, framework, or tag
- `get_pattern` - Get specific pattern by name
- `create_pattern` - Create new pattern with metadata and content

## Pattern Browser TUI

The project includes a terminal-based pattern browser (`browse`) as a separate binary target. It lets you browse, search, and read your patterns directly in the terminal without needing an MCP client.

### Running the Browser

```bash
PATTERNS_DIR="/path/to/your/patterns" cargo run --bin browse
```

Or build and run the release binary:

```bash
cargo build --release
PATTERNS_DIR="/path/to/your/patterns" ./target/release/browse
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `/` | Search / filter patterns |
| `Esc` | Cancel search |
| `Enter` | Apply search filter |
| `PgUp` / `PgDn` | Scroll pattern content |
| `q` | Quit |

## Building

```bash
cargo build --release
```

This produces two binaries:
- `./target/release/grimoire-mcp` — the MCP server
- `./target/release/browse` — the TUI pattern browser

## MCP Client Configuration

### Kiro CLI

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "pattern-library": {
      "command": "/path/to/grimoire-mcp/target/release/grimoire-mcp",
      "env": {
        "PATTERNS_DIR": "/path/to/your/patterns"
      }
    }
  }
}
```

### Cursor IDE

Add to your Cursor settings:

```json
{
  "mcp": {
    "servers": {
      "pattern-library": {
        "command": "/path/to/grimoire-mcp/target/release/grimoire-mcp",
        "env": {
          "PATTERNS_DIR": "/path/to/your/patterns"
        }
      }
    }
  }
}
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or equivalent:

```json
{
  "mcpServers": {
    "pattern-library": {
      "command": "/path/to/grimoire-mcp/target/release/grimoire-mcp",
      "env": {
        "PATTERNS_DIR": "/path/to/your/patterns"
      }
    }
  }
}
```

## MCP Server Development

```bash
# Set patterns directory
export PATTERNS_DIR="/home/darko/workspace/kiro-projects/better-agent/pattern-library/patterns"

# Run in debug mode
just debug

# Test with MCP inspector
just mcp-test
```
