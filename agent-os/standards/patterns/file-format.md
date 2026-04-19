# Pattern File Format

YAML frontmatter + markdown body.

```markdown
---
pattern: error-handling
category: rust
framework: axum
projects: [project1, project2]
tags: [web, api, error-handling]
---
Pattern explanation and code here...
```

| Field | Type | Required |
|-------|------|----------|
| `pattern` | `String` | yes |
| `category` | `String` | yes |
| `framework` | `Option<String>` | no |
| `projects` | `Vec<String>` | no (serde default) |
| `tags` | `Vec<String>` | no (serde default) |
