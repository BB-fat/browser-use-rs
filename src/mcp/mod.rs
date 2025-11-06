//! MCP (Model Context Protocol) server implementation for browser automation
//!
//! This module provides rmcp-compatible tools by wrapping the existing tool implementations.

pub mod handler;
pub use handler::BrowserServer;

use crate::tools::{ToolContext, ToolResult as InternalToolResult};
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Navigate tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NavigateParams {
    /// URL to navigate to
    pub url: String,
    /// Wait for navigation to complete (default: true)
    #[serde(default = "default_true")]
    pub wait_for_load: bool,
}

/// Click tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClickParams {
    /// CSS selector of the element to click
    #[serde(default)]
    pub selector: Option<String>,
    /// Element index from DOM tree
    #[serde(default)]
    pub index: Option<usize>,
}

/// Input tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputParams {
    /// CSS selector of the input element
    #[serde(default)]
    pub selector: Option<String>,
    /// Element index from DOM tree
    #[serde(default)]
    pub index: Option<usize>,
    /// Text to input
    pub text: String,
}

/// Extract content parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractParams {
    /// CSS selector to extract content from (optional)
    #[serde(default)]
    pub selector: Option<String>,
}

/// Screenshot parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotParams {
    /// Whether to capture full page (default: false)
    #[serde(default)]
    pub full_page: bool,
}

/// JavaScript evaluation parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvaluateParams {
    /// JavaScript code to execute
    pub script: String,
}

/// Wait parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaitParams {
    /// Duration in milliseconds
    pub duration_ms: u64,
}

fn default_true() -> bool {
    true
}

/// Convert internal ToolResult to MCP CallToolResult
fn convert_result(result: InternalToolResult) -> Result<CallToolResult, McpError> {
    if result.success {
        let text = if let Some(data) = result.data {
            serde_json::to_string_pretty(&data).unwrap_or_else(|_| data.to_string())
        } else {
            "Success".to_string()
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    } else {
        let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
        Err(McpError::internal_error(error_msg, None))
    }
}

#[tool_router]
impl BrowserServer {
    /// Navigate to a URL
    #[tool(description = "Navigate to a specified URL in the browser")]
    fn browser_navigate(
        &self,
        params: Parameters<NavigateParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let tool_params = serde_json::json!({
            "url": params.0.url,
            "wait_for_load": params.0.wait_for_load
        });

        let result = session
            .tool_registry()
            .execute("navigate", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }

    /// Click on an element
    #[tool(description = "Click on an element specified by CSS selector or index")]
    fn browser_click(&self, params: Parameters<ClickParams>) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let tool_params = if let Some(selector) = params.0.selector {
            serde_json::json!({ "selector": selector })
        } else if let Some(index) = params.0.index {
            serde_json::json!({ "index": index })
        } else {
            return Err(McpError::invalid_params(
                "Either selector or index must be provided",
                None,
            ));
        };

        let result = session
            .tool_registry()
            .execute("click", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }

    /// Fill input field with text
    #[tool(description = "Fill an input field with text")]
    fn browser_form_input_fill(
        &self,
        params: Parameters<InputParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let mut tool_params = serde_json::json!({
            "text": params.0.text
        });

        if let Some(selector) = params.0.selector {
            tool_params["selector"] = serde_json::json!(selector);
        } else if let Some(index) = params.0.index {
            tool_params["index"] = serde_json::json!(index);
        } else {
            return Err(McpError::invalid_params(
                "Either selector or index must be provided",
                None,
            ));
        }

        let result = session
            .tool_registry()
            .execute("input", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }

    /// Extract text content from the page
    #[tool(description = "Extract text content from the page or a specific element")]
    fn browser_get_text(
        &self,
        params: Parameters<ExtractParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let tool_params = if let Some(selector) = params.0.selector {
            serde_json::json!({ "selector": selector })
        } else {
            serde_json::json!({})
        };

        let result = session
            .tool_registry()
            .execute("extract_content", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }

    /// Take a screenshot of the page
    #[tool(description = "Take a screenshot of the current page")]
    fn browser_screenshot(
        &self,
        params: Parameters<ScreenshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let tool_params = serde_json::json!({
            "full_page": params.0.full_page
        });

        let result = session
            .tool_registry()
            .execute("screenshot", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }

    /// Evaluate JavaScript code on the page
    #[tool(description = "Execute JavaScript code in the browser context")]
    fn browser_evaluate(
        &self,
        params: Parameters<EvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let tool_params = serde_json::json!({
            "script": params.0.script
        });

        let result = session
            .tool_registry()
            .execute("evaluate", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }

    /// Wait for a specified duration
    #[tool(description = "Wait for a specified duration in milliseconds")]
    fn browser_wait(&self, params: Parameters<WaitParams>) -> Result<CallToolResult, McpError> {
        let session = self.session();
        let mut context = ToolContext::new(&*session);

        let tool_params = serde_json::json!({
            "duration_ms": params.0.duration_ms
        });

        let result = session
            .tool_registry()
            .execute("wait", tool_params, &mut context)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        convert_result(result)
    }
}
