use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub struct InputTool;

impl Tool for InputTool {
    fn name(&self) -> &str {
        "input"
    }

    fn description(&self) -> &str {
        "Type text into an input element"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(InputParams)).unwrap_or_default()
    }

    fn execute(&self, params: Value, context: &mut ToolContext) -> Result<ToolResult> {
        let params: InputParams = serde_json::from_value(params)
            .map_err(|e| BrowserError::InvalidArgument(e.to_string()))?;

        let element = context.session.find_element(&params.selector)?;
        
        if params.clear {
            element.click().ok(); // Focus
            // Clear with Ctrl+A and Delete
            context.session.tab().press_key("End").ok();
            for _ in 0..params.text.len() + 100 {
                context.session.tab().press_key("Backspace").ok();
            }
        }
        
        element.type_into(&params.text)
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
