use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputParams {
    /// CSS selector (use either this or index, not both)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// Element index from DOM tree (use either this or selector, not both)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

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
        // Validate that exactly one selector method is provided
        match (&params.selector, &params.index) {
            (Some(_), Some(_)) => {
                return Err(BrowserError::ToolExecutionFailed {
                    tool: "input".to_string(),
                    reason: "Cannot specify both 'selector' and 'index'. Use one or the other."
                        .to_string(),
                });
            }
            (None, None) => {
                return Err(BrowserError::ToolExecutionFailed {
                    tool: "input".to_string(),
                    reason: "Must specify either 'selector' or 'index'.".to_string(),
                });
            }
            _ => {}
        }

        // Get the CSS selector (either directly or from index)
        let css_selector = if let Some(selector) = params.selector.clone() {
            selector
        } else if let Some(index) = params.index {
            let dom = context.get_dom()?;
            let selector = dom.get_selector(index).ok_or_else(|| {
                BrowserError::ElementNotFound(format!("No element with index {}", index))
            })?;
            selector.clone()
        } else {
            unreachable!("Validation above ensures one field is Some")
        };

        let tab = context.session.tab()?;
        let element = context.session.find_element(&tab, &css_selector)?;

        if params.clear {
            element.click().ok(); // Focus
            // Clear with Ctrl+A and Delete
            tab.press_key("End").ok();
            for _ in 0..params.text.len() + 100 {
                tab.press_key("Backspace").ok();
            }
        }

        element
            .type_into(&params.text)
            .map_err(|e| BrowserError::ToolExecutionFailed {
                tool: "input".to_string(),
                reason: e.to_string(),
            })?;

        let result_json = if let Some(index) = params.index {
            serde_json::json!({
                "index": index,
                "selector": css_selector,
                "text_length": params.text.len(),
                "method": "index"
            })
        } else {
            serde_json::json!({
                "selector": css_selector,
                "text_length": params.text.len(),
                "method": "css"
            })
        };

        Ok(ToolResult::success_with(result_json))
    }
}
