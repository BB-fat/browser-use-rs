use crate::error::Result;
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parameters for the navigate tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NavigateParams {
    /// URL to navigate to
    pub url: String,
    
    /// Wait for navigation to complete (default: true)
    #[serde(default = "default_wait")]
    pub wait_for_load: bool,
}

fn default_wait() -> bool {
    true
}

/// Tool for navigating to a URL
pub struct NavigateTool;

impl Tool for NavigateTool {
    fn name(&self) -> &str {
        "navigate"
    }

    fn description(&self) -> &str {
        "Navigate to a specified URL in the browser"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(NavigateParams)).unwrap_or_default()
    }

    fn execute(&self, params: Value, context: &mut ToolContext) -> Result<ToolResult> {
        // Parse parameters
        let params: NavigateParams = serde_json::from_value(params)
            .map_err(|e| crate::error::BrowserError::InvalidArgument(format!("Invalid navigate parameters: {}", e)))?;

        // Navigate to URL
        context.session.navigate(&params.url)?;

        // Wait for navigation if requested
        if params.wait_for_load {
            context.session.wait_for_navigation()?;
        }

        Ok(ToolResult::success_with(serde_json::json!({
            "url": params.url,
            "waited": params.wait_for_load
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigate_params_default() {
        let json = serde_json::json!({
            "url": "https://example.com"
        });

        let params: NavigateParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert!(params.wait_for_load);
    }

    #[test]
    fn test_navigate_params_explicit_wait() {
        let json = serde_json::json!({
            "url": "https://example.com",
            "wait_for_load": false
        });

        let params: NavigateParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert!(!params.wait_for_load);
    }

    #[test]
    fn test_navigate_tool_metadata() {
        let tool = NavigateTool;
        assert_eq!(tool.name(), "navigate");
        assert!(!tool.description().is_empty());
        
        let schema = tool.parameters_schema();
        assert!(schema.is_object());
    }
}
