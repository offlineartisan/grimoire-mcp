use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use rmcp::{
    RoleServer, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, InitializeRequestParam, InitializeResult,
        ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

use rmcp::ErrorData as McpError;

const ENV_PATTERNS_DIR: &str = "PATTERNS_DIR";

#[derive(Debug, Clone)]
pub struct Patterns {
    patterns: Arc<Vec<Pattern>>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pattern {
    metadata: PatternMetadata,
    content: String,
    filepath: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatternMetadata {
    pattern: String,
    category: String,
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

// === Request structs ===

/// Search parameters
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatternSearchRequest {
    #[schemars(description = "Text Search")]
    query: Option<String>,
    #[schemars(description = "Filter by category")]
    category: Option<String>,
    #[schemars(description = "Filter by framework")]
    framework: Option<String>,
    #[schemars(description = "Filter by tag")]
    tag: Option<String>,
}

/// Get parameters
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPatternRequest {
    #[schemars(description = "Pattern Name")]
    pattern_name: String,
}

/// Create parameters
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreatePatternRequest {
    #[schemars(description = "Pattern name")]
    pattern_name: String,
    #[schemars(description = "Pattern category")]
    category: String,
    #[schemars(description = "Pattern framework")]
    framework: String,
    #[schemars(description = "Projects in which these patterns were used")]
    projects: Option<Vec<String>>,
    #[schemars(description = "Pattern tags")]
    tag: Vec<String>,
    #[schemars(description = "Pattern content")]
    content: String,
}

impl Patterns {
    /// Parse pattern from file
    fn load_patterns(path: &Path) -> Option<Pattern> {
        let content = fs::read_to_string(path).ok()?;
        let rest = content.strip_prefix("---\n")?;
        let mut parts = rest.splitn(2, "\n---\n");
        let yaml = parts.next()?;
        let body = parts.next()?.trim();
        let metadata: PatternMetadata = serde_yaml::from_str(yaml).ok()?;

        Some(Pattern {
            metadata,
            content: body.to_string(),
            filepath: path.to_path_buf(),
        })
    }

    /// Load patterns from the provided directory
    fn load_all_patterns() -> Vec<Pattern> {
        let patterns_dir =
            std::env::var(ENV_PATTERNS_DIR).expect("PATTERNS_DIR environment variable MUST be set");
        let patterns_dir = PathBuf::from(patterns_dir);

        fs::read_dir(&patterns_dir)
            .ok()
            .into_iter()
            .flatten()              // Extract good ReadDir
            .flat_map(|e| e.ok())   // Convet Result<DireEntry, Err> to DirEntry
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .filter_map(|e| Self::load_patterns(&e.path()))
            .collect()
    }
    /// Validate the pattern name during creation
    fn validate_pattern_name(name: &str) -> Result<(), McpError> {
        if name.is_empty() || name.len() > 100 {
            return Err(McpError::invalid_params(
                "Pattern must be 1-100 characters",
                None,
            ));
        }
        if name
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
        {
            return Err(McpError::invalid_params(
                "Pattern name can only contain alphanumeric, dash and underscore characters",
                None,
            ));
        }

        Ok(())
    }

}

#[tool_router]
impl Patterns {
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(Self::load_all_patterns()),
            tool_router: Self::tool_router(),
        }
    }

    /// Get all available patterns
    #[tool(description = "List all available patterns")]
    fn list_patterns(&self) -> Result<CallToolResult, McpError> {
        let summary: Vec<String> = self
            .patterns
            .iter()
            .map(|p| format!("- {} ({})", p.metadata.pattern, p.metadata.category))
            .collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Available patterns:\n{}",
            summary.join("\n")
        ))]))
    }

    /// Search patterns based on input
    #[tool(description = "Search patterns by query, category, framework or tag")]
    fn search_patterns(
        &self,
        Parameters(PatternSearchRequest {
            query,
            category,
            framework,
            tag,
        }): Parameters<PatternSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let results: Vec<&Pattern> = self
            .patterns
            .iter()
            .filter(|p| { // Search through the fields
                category.as_ref().is_none_or(|c| &p.metadata.category == c)
                    && framework
                        .as_ref()
                        .is_none_or(|f| p.metadata.framework.as_ref() == Some(f))
                    && tag.as_ref().is_none_or(|t| p.metadata.tags.contains(t))
                    && query.as_ref().is_none_or(|q| { 
                    // Match query to the pattern name and content
                        let searchable =
                            format!("{} {}", p.metadata.pattern, p.content).to_lowercase();
                        searchable.contains(&q.to_lowercase())
                    })
            })
            .collect();

        if results.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No patterns found.",
            )]));
        }

        let summary: Vec<String> = results
            .iter()
            .map(|p| {
                let max_bytes = 200.min(p.content.len());
                let truncate_at = p
                    .content
                    .char_indices()
                    .map(|(i, _)| i)
                    .take_while(|&i| i <= max_bytes)
                    .last()
                    .unwrap_or(0);
                format!(
                    "**{}**\n{}",
                    p.metadata.pattern,
                    &p.content[..truncate_at]
                )
            })
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            summary.join("\n\n"),
        )]))
    }

    /// Get the pattern based on the name
    #[tool(description = "Get the pattern based on the pattern name")]
    fn get_pattern(
        &self,
        Parameters(GetPatternRequest { pattern_name }): Parameters<GetPatternRequest>,
    ) -> Result<CallToolResult, McpError> {
        let pattern = self
            .patterns
            .iter()
            .find(|p| p.metadata.pattern == pattern_name);

        match pattern {
            Some(p) => Ok(CallToolResult::success(vec![Content::text(&p.content)])),
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "Pattern '{}' not found.",
                pattern_name
            ))])),
        }
    }

    /// Create patterns by providing information
    #[tool(
        description = "Create patterns by providing, category, framework, projects this pattern was used in, tags, and the content. Look to existing patterns for examples on how this should look"
    )]
    fn create_pattern(
        &self,
        Parameters(CreatePatternRequest {
            pattern_name,
            category,
            framework,
            projects,
            tag,
            content,
        }): Parameters<CreatePatternRequest>,
    ) -> Result<CallToolResult, McpError> {
        let projects_str = projects
            .map(|p| format!("projects: [{}]\n", p.join(", ")))
            .unwrap_or_default();

        let tags_str = if tag.is_empty() {
            String::new()
        } else {
            format!("tags: [{}]\n", tag.join(", "))
        };

        // Validate Name
        Self::validate_pattern_name(&pattern_name)?;

        let pattern_content = format!(
            r#"---
pattern: {}
category: {}
framework: {}
{}{}---

{}
"#,
            pattern_name, category, framework, projects_str, tags_str, content
        );

        let patterns_dir =
            std::env::var(ENV_PATTERNS_DIR).expect("PATTERNS_DIR environment variable MUST be set");
        let file_path = PathBuf::from(patterns_dir).join(format!("{}.md", pattern_name));

        match fs::write(&file_path, pattern_content) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Pattern '{}' created at {:?}",
                pattern_name, file_path
            ))])),
            Err(e) => Err(McpError::internal_error(
                format!("Failed to create pattern: {}", e),
                None,
            )),
        }
    }
}

#[tool_handler]
impl ServerHandler for Patterns {
    /// Provide server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
    "I manage a library of software development patterns stored as markdown files with YAML frontmatter.
    Use me to discover, search, and create reusable code patterns and architectural solutions.

    Available operations:
    - list_patterns: Get overview of all available patterns
    - search_patterns: Find patterns by text, category, framework, or tags
    - get_pattern: Retrieve full content of a specific pattern
    - create_pattern: Add new patterns with proper metadata

    Patterns include categories like 'rust', 'aws', 'web' and frameworks like 'axum', 'lambda'.
    Each pattern contains implementation details, best practices, and usage examples.

    When creating patterns, include relevant tags and specify which projects used them for better discoverability.".to_string()
),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
