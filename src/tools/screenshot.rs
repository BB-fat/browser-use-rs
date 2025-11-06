use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotParams {
    /// Path to save the screenshot
    pub path: String,
    
    /// Capture full page (default: false)
    #[serde(default)]
    pub full_page: bool,
}

pub struct ScreenshotTool;

impl Tool for ScreenshotTool {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> &str {
        "Capture a screenshot of the current page"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ScreenshotParams)).unwrap_or_default()
    }

    fn execute(&self, params: Value, context: &mut ToolContext) -> Result<ToolResult> {
        let params: ScreenshotParams = serde_json::from_value(params)
            .map_err(|e| BrowserError::InvalidArgument(e.to_string()))?;

        let screenshot_data = context.session.tab()
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                params.full_page,
            )
            .map_err(|e| BrowserError::ScreenshotFailed(e.to_string()))?;

        std::fs::write(&params.path, &screenshot_data)
            .map_err(|e| BrowserError::ScreenshotFailed(format!("Failed to save screenshot: {}", e)))?;

        Ok(ToolResult::success_with(serde_json::json!({
            "path": params.path,
            "size_bytes": screenshot_data.len(),
            "full_page": params.full_page
        })))
    }
}
