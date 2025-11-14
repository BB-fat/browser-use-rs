use crate::error::{BrowserError, Result};
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for the select tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SelectParams {
    /// Element selector (CSS selector or index)
    #[serde(flatten)]
    pub selector: ElementSelector,

    /// Value to select in the dropdown
    pub value: String,
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

/// Tool for selecting dropdown options
#[derive(Default)]
pub struct SelectTool;

const SELECT_JS: &str = include_str!("select.js");

impl Tool for SelectTool {
    type Params = SelectParams;

    fn name(&self) -> &str {
        "select"
    }

    fn execute_typed(&self, params: SelectParams, context: &mut ToolContext) -> Result<ToolResult> {
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
        let value = params.value;

        let select_config = serde_json::json!({
            "selector": css_selector,
            "value": value,
        });
        let select_js = SELECT_JS.replace("__SELECT_CONFIG__", &select_config.to_string());

        let result = context
            .session
            .tab()
            .evaluate(&select_js, false)
            .map_err(|e| BrowserError::ToolExecutionFailed {
                tool: "select".to_string(),
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
                "value": value,
                "selectedText": result_json["selectedText"]
            })))
        } else {
            Err(BrowserError::ToolExecutionFailed {
                tool: "select".to_string(),
                reason: result_json["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_params_css() {
        let json = serde_json::json!({
            "selector": "#country-select",
            "value": "us"
        });

        let params: SelectParams = serde_json::from_value(json).unwrap();
        match params.selector {
            ElementSelector::Css { selector } => assert_eq!(selector, "#country-select"),
            _ => panic!("Expected CSS selector"),
        }
        assert_eq!(params.value, "us");
    }

    #[test]
    fn test_select_params_index() {
        let json = serde_json::json!({
            "index": 5,
            "value": "option2"
        });

        let params: SelectParams = serde_json::from_value(json).unwrap();
        match params.selector {
            ElementSelector::Index { index } => assert_eq!(index, 5),
            _ => panic!("Expected index selector"),
        }
        assert_eq!(params.value, "option2");
    }
}
