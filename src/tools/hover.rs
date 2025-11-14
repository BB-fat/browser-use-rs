use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for the hover tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HoverParams {
    /// Element selector (CSS selector or index)
    #[serde(flatten)]
    pub selector: ElementSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ElementSelector {
    /// Select by CSS selector
    Css {
        /// CSS selector
        selector: String,
    },
    /// Select by index from DOM tree
    Index {
        /// Element index
        index: usize,
    },
}

/// Tool for hovering over elements
#[derive(Default)]
pub struct HoverTool;

const HOVER_JS: &str = include_str!("hover.js");

impl Tool for HoverTool {
    type Params = HoverParams;

    fn name(&self) -> &str {
        "hover"
    }

    fn execute_typed(&self, params: HoverParams, context: &mut ToolContext) -> Result<ToolResult> {
        let css_selector = match params.selector {
            ElementSelector::Css { selector } => selector,
            ElementSelector::Index { index } => {
                let dom = context.get_dom()?;
                let selector_info = dom.get_selector(index).ok_or_else(|| {
                    BrowserError::ElementNotFound(format!("No element with index {}", index))
                })?;
                selector_info.css_selector.clone()
            }
        };

        // Find the element (to verify it exists)

        // Scroll into view if needed, then hover
        let selector_json =
            serde_json::to_string(&css_selector).expect("serializing CSS selector never fails");
        let hover_js = HOVER_JS.replace("__SELECTOR__", &selector_json);

        let result = context
            .session
            .tab()
            .evaluate(&hover_js, false)
            .map_err(|e| BrowserError::ToolExecutionFailed {
                tool: "hover".to_string(),
                reason: e.to_string(),
            })?;

        // Parse the JSON string returned by JavaScript
        let result_json: serde_json::Value = if let Some(serde_json::Value::String(json_str)) =
            result.value
        {
            serde_json::from_str(&json_str)
                .unwrap_or(serde_json::json!({"success": false, "error": "Failed to parse result"}))
        } else {
            result
                .value
                .unwrap_or(serde_json::json!({"success": false, "error": "No result returned"}))
        };

        if result_json["success"].as_bool() == Some(true) {
            Ok(ToolResult::success_with(serde_json::json!({
                "selector": css_selector,
                "element": {
                    "tagName": result_json["tagName"],
                    "id": result_json["id"],
                    "className": result_json["className"]
                }
            })))
        } else {
            Err(BrowserError::ToolExecutionFailed {
                tool: "hover".to_string(),
                reason: result_json["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            })
        }
    }
}
