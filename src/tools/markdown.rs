use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for getting markdown content (no parameters needed)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetMarkdownParams {}

#[derive(Default)]
pub struct GetMarkdownTool;

impl Tool for GetMarkdownTool {
    type Params = GetMarkdownParams;

    fn name(&self) -> &str {
        "get_markdown"
    }

    fn execute_typed(
        &self,
        _params: GetMarkdownParams,
        context: &mut ToolContext,
    ) -> Result<ToolResult> {
        // Wait for network idle with a timeout (similar to TypeScript version)
        // Since headless_chrome doesn't have a direct network idle wait,
        // we'll add a small delay to let dynamic content load
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Load the JavaScript code for markdown conversion
        let js_code = include_str!("convert_to_markdown.js");

        // Execute the JavaScript to extract and convert content
        let result = context
            .session
            .tab()
            .evaluate(js_code, false)
            .map_err(|e| BrowserError::EvaluationFailed(e.to_string()))?;

        // Parse the result
        let result_value = result
            .value
            .ok_or_else(|| BrowserError::ToolExecutionFailed {
                tool: "get_markdown".to_string(),
                reason: "No value returned from JavaScript".to_string(),
            })?;

        // The JavaScript returns a JSON string, so we need to parse it
        let content_data: MarkdownContent = if let Some(json_str) = result_value.as_str() {
            serde_json::from_str(json_str).map_err(|e| BrowserError::ToolExecutionFailed {
                tool: "get_markdown".to_string(),
                reason: format!("Failed to parse markdown content: {}", e),
            })?
        } else {
            // If it's already an object, try to deserialize directly
            serde_json::from_value(result_value).map_err(|e| BrowserError::ToolExecutionFailed {
                tool: "get_markdown".to_string(),
                reason: format!("Failed to deserialize markdown content: {}", e),
            })?
        };

        // Combine title and content
        let markdown = if !content_data.title.is_empty() {
            format!("# {}\n\n{}", content_data.title, content_data.content)
        } else {
            content_data.content.clone()
        };

        Ok(ToolResult::success_with(serde_json::json!({
            "markdown": markdown,
            "title": content_data.title,
            "url": content_data.url,
            "length": markdown.len()
        })))
    }
}

/// Structure for markdown content returned from JavaScript
#[derive(Debug, Serialize, Deserialize)]
struct MarkdownContent {
    title: String,
    content: String,
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_markdown_tool_name() {
        let tool = GetMarkdownTool::default();
        assert_eq!(tool.name(), "get_markdown");
    }

    #[test]
    fn test_get_markdown_params_schema() {
        let tool = GetMarkdownTool::default();
        let schema = tool.parameters_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_markdown_content_deserialization() {
        let json = r#"{"title": "Test", "content": "Hello", "url": "https://example.com"}"#;
        let content: MarkdownContent = serde_json::from_str(json).unwrap();
        assert_eq!(content.title, "Test");
        assert_eq!(content.content, "Hello");
        assert_eq!(content.url, "https://example.com");
    }
}
