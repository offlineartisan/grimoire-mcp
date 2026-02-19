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

## Building

```bash
cargo build --release
```

The compiled binary will be at `./target/release/grimoire-mcp`.

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
