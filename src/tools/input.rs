use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputParams {
    /// CSS selector for the input element
    pub selector: String,

    /// Text to type into the element
    pub text: String,

    /// Clear existing content first (default: false)
    #[serde(default)]
    pub clear: bool,
}

#[derive(Default)]
pub struct InputTool;

impl Tool for InputTool {
    type Params = InputParams;

    fn name(&self) -> &str {
        "input"
    }

    fn execute_typed(&self, params: InputParams, context: &mut ToolContext) -> Result<ToolResult> {
        let element = context.session.find_element(&params.selector)?;

        if params.clear {
            element.click().ok(); // Focus
            // Clear with Ctrl+A and Delete
            context.session.tab().press_key("End").ok();
            for _ in 0..params.text.len() + 100 {
                context.session.tab().press_key("Backspace").ok();
            }
        }

        element
            .type_into(&params.text)
            .map_err(|e| BrowserError::ToolExecutionFailed {
                tool: "input".to_string(),
                reason: e.to_string(),
            })?;

        Ok(ToolResult::success_with(serde_json::json!({
            "selector": params.selector,
            "text_length": params.text.len()
        })))
    }
}
