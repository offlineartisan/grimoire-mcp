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
    pub metadata: PatternMetadata,
    pub content: String,
    pub filepath: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatternMetadata {
    pub pattern: String,
    pub category: String,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
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

/// Parse a single pattern from a markdown file with YAML frontmatter.
pub fn load_pattern(path: &Path) -> Option<Pattern> {
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

/// Load all patterns from the `PATTERNS_DIR` environment variable directory.
pub fn load_all_patterns() -> Vec<Pattern> {
    let patterns_dir =
        std::env::var(ENV_PATTERNS_DIR).expect("PATTERNS_DIR environment variable MUST be set");
    let patterns_dir = PathBuf::from(patterns_dir);

    fs::read_dir(&patterns_dir)
        .ok()
        .into_iter()
        .flatten()              // Extract good ReadDir
        .flat_map(|e| e.ok())   // Convert Result<DirEntry, Err> to DirEntry
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| load_pattern(&e.path()))
        .collect()
}

impl Patterns {
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
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '_' && c != ' ')
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
            patterns: Arc::new(load_all_patterns()),
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

    /// Sanitize a pattern name for use as a filename.
    /// Replaces spaces with dashes and lowercases the result.
    fn sanitize_filename(name: &str) -> String {
        name.replace(' ', "-").to_lowercase()
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
        let file_path = PathBuf::from(patterns_dir).join(format!("{}.md", Self::sanitize_filename(&pattern_name)));

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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy: generate pattern names that contain at least one space,
    /// with all characters drawn from [a-zA-Z0-9 _-], length 1..=100.
    fn space_containing_pattern_name() -> impl Strategy<Value = String> {
        // Generate a base string from the valid charset (including space)
        prop::string::string_regex("[a-zA-Z0-9 _-]{1,100}")
            .unwrap()
            .prop_filter("must contain at least one space", |s| {
                s.contains(' ') && !s.is_empty() && s.len() <= 100
            })
    }

    /// Helper: what a sanitize_filename function SHOULD produce.
    /// Encodes the expected behavior from the design doc.
    fn expected_sanitized_filename(name: &str) -> String {
        name.replace(' ', "-").to_lowercase()
    }

    proptest! {
        /// Property 1 (Bug Condition): Space-containing pattern names should be accepted.
        ///
        /// This test MUST FAIL on unfixed code — failure confirms the bug exists.
        /// DO NOT fix the test or the code when it fails.
        ///
        /// When the fix is applied, this test will pass, confirming the expected behavior.
        #[test]
        fn bug_condition_spaces_accepted(name in space_containing_pattern_name()) {
            // The bug: validate_pattern_name rejects spaces.
            // Expected behavior: it should accept them.
            let result = Patterns::validate_pattern_name(&name);
            prop_assert!(
                result.is_ok(),
                "validate_pattern_name rejected '{}': {:?}",
                name,
                result.unwrap_err()
            );
        }

        /// Property 1b (Expected Behavior): Sanitized filename should have no spaces
        /// and be lowercased, while the original display name is preserved.
        ///
        /// This encodes the design requirement that create_pattern produces a sanitized
        /// filename (spaces→dashes, lowercased) and preserves the original name in YAML.
        #[test]
        fn bug_condition_sanitized_filename_spec(name in space_containing_pattern_name()) {
            let sanitized = expected_sanitized_filename(&name);

            // Sanitized filename must not contain spaces
            prop_assert!(
                !sanitized.contains(' '),
                "sanitized filename '{}' still contains spaces (from '{}')",
                sanitized,
                name
            );

            // Sanitized filename must be lowercase
            prop_assert!(
                sanitized.chars().all(|c| !c.is_uppercase()),
                "sanitized filename '{}' is not fully lowercased",
                sanitized
            );

            // Original display name is preserved (not equal to sanitized when it has spaces/uppercase)
            if name.contains(' ') || name.chars().any(|c| c.is_uppercase()) {
                prop_assert_ne!(
                    name, sanitized,
                    "display name should differ from sanitized filename when spaces or uppercase present"
                );
            }
        }
    }

    /// Deterministic unit test confirming the bug with a known input.
    #[test]
    fn bug_condition_unit_error_handling_pattern() {
        let result = Patterns::validate_pattern_name("Error Handling Pattern");
        // On unfixed code this will be Err — confirming the bug.
        // After fix this should be Ok.
        assert!(
            result.is_ok(),
            "validate_pattern_name rejected 'Error Handling Pattern': {:?}",
            result.unwrap_err()
        );
    }

    /// Deterministic unit test confirming the bug with another known input.
    #[test]
    fn bug_condition_unit_builder_pattern() {
        let result = Patterns::validate_pattern_name("Builder Pattern");
        assert!(
            result.is_ok(),
            "validate_pattern_name rejected 'Builder Pattern': {:?}",
            result.unwrap_err()
        );
    }

    // =========================================================================
    // Property 2: Preservation — Non-Space Pattern Name Behavior Unchanged
    // =========================================================================
    //
    // These tests observe and lock down the CURRENT behavior of validate_pattern_name
    // for inputs that do NOT contain spaces. They must pass on UNFIXED code and
    // continue to pass after the fix (no regressions).

    // --- Observation unit tests (confirm current behavior before writing PBTs) ---

    /// Observation: valid name without spaces is accepted.
    #[test]
    fn preservation_observe_valid_name_accepted() {
        let result = Patterns::validate_pattern_name("error-handling");
        assert!(result.is_ok(), "expected Ok for 'error-handling', got: {:?}", result);
    }

    /// Observation: empty name is rejected.
    #[test]
    fn preservation_observe_empty_rejected() {
        let result = Patterns::validate_pattern_name("");
        assert!(result.is_err(), "expected Err for empty name, got Ok");
    }

    /// Observation: special character (']') is rejected.
    #[test]
    fn preservation_observe_special_char_rejected() {
        let result = Patterns::validate_pattern_name("a]b");
        assert!(result.is_err(), "expected Err for 'a]b', got Ok");
    }

    /// Observation: name exceeding 100 chars is rejected.
    #[test]
    fn preservation_observe_too_long_rejected() {
        let long_name = "a".repeat(101);
        let result = Patterns::validate_pattern_name(&long_name);
        assert!(result.is_err(), "expected Err for 101-char name, got Ok");
    }

    // --- Property-based preservation tests ---

    proptest! {
        /// Preservation PBT 1: All strings from [a-zA-Z0-9_-]{1,100} (no spaces)
        /// must be accepted by validate_pattern_name.
        #[test]
        fn preservation_valid_no_space_names_accepted(
            name in prop::string::string_regex("[a-zA-Z0-9_-]{1,100}").unwrap()
        ) {
            let result = Patterns::validate_pattern_name(&name);
            prop_assert!(
                result.is_ok(),
                "validate_pattern_name rejected valid no-space name '{}': {:?}",
                name,
                result.unwrap_err()
            );
        }

        /// Preservation PBT 2: Any string containing at least one character NOT in
        /// [a-zA-Z0-9 _-] must be rejected by validate_pattern_name.
        ///
        /// Strategy: generate a 1-100 char string from the full printable ASCII range,
        /// then filter to those containing at least one "illegal" character.
        #[test]
        fn preservation_invalid_chars_rejected(
            name in prop::string::string_regex("[\\x20-\\x7E]{1,100}")
                .unwrap()
                .prop_filter(
                    "must contain at least one char outside [a-zA-Z0-9 _-]",
                    |s| s.chars().any(|c| !c.is_alphanumeric() && c != '-' && c != '_' && c != ' ')
                )
        ) {
            let result = Patterns::validate_pattern_name(&name);
            prop_assert!(
                result.is_err(),
                "validate_pattern_name accepted '{}' which contains invalid characters",
                name
            );
        }

        /// Preservation PBT 3: Empty strings must be rejected.
        /// (proptest can't generate empty from regex easily, so this is a unit-style prop)
        #[test]
        fn preservation_too_long_names_rejected(
            name in prop::string::string_regex("[a-zA-Z0-9_-]{101,200}").unwrap()
        ) {
            let result = Patterns::validate_pattern_name(&name);
            prop_assert!(
                result.is_err(),
                "validate_pattern_name accepted '{}' (len={}) which exceeds 100 chars",
                name,
                name.len()
            );
        }
    }
}
