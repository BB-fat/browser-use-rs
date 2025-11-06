use crate::error::Result;
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for the get_clickable_elements tool (no parameters needed)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetClickableElementsParams {}

/// Tool for getting all clickable/interactive elements on the page
#[derive(Default)]
pub struct GetClickableElementsTool;

impl Tool for GetClickableElementsTool {
    type Params = GetClickableElementsParams;

    fn name(&self) -> &str {
        "get_clickable_elements"
    }

    fn execute_typed(
        &self,
        _params: GetClickableElementsParams,
        context: &mut ToolContext,
    ) -> Result<ToolResult> {
        // Get or extract the DOM tree
        let dom = context.get_dom()?;

        // Get all interactive element indices
        let indices = dom.interactive_indices();

        if indices.is_empty() {
            return Ok(ToolResult::success_with(serde_json::json!({
                "elements": "",
                "count": 0
            })));
        }

        // Format clickable elements similar to TypeScript implementation:
        // [0]<button>Submit</button>
        // [1]<a>Click here</a>
        // [2]<input>
        let mut formatted_elements = Vec::new();

        for &index in &indices {
            if let Some(node) = dom.find_node_by_index(index) {
                let tag_name = &node.tag_name;

                // Get text content, truncate if too long
                let text_content = if let Some(text) = &node.text_content {
                    let trimmed = text.trim();
                    if trimmed.len() > 100 {
                        format!("{}...", &trimmed[..97])
                    } else {
                        trimmed.to_string()
                    }
                } else {
                    String::new()
                };

                // Format: [index]<tag>text</tag>
                let formatted = if text_content.is_empty() {
                    format!("[{}]<{}>", index, tag_name)
                } else {
                    format!("[{}]<{}>{}</{}>", index, tag_name, text_content, tag_name)
                };

                formatted_elements.push(formatted);
            }
        }

        let elements_string = formatted_elements.join("\n");
        let count = indices.len();

        Ok(ToolResult::success_with(serde_json::json!({
            "elements": elements_string,
            "count": count
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::DomTree;
    use crate::dom::ElementNode;

    #[test]
    fn test_get_clickable_elements_params() {
        let params = GetClickableElementsParams {};
        let json = serde_json::to_value(params).unwrap();
        assert!(json.is_object());
    }

    #[test]
    fn test_tool_name() {
        let tool = GetClickableElementsTool;
        assert_eq!(tool.name(), "get_clickable_elements");
    }

    // Note: Full integration tests would require a real browser session
    // This is a basic structure test
    #[test]
    fn test_empty_dom_tree() {
        // Create a minimal DOM tree with no interactive elements
        let root = ElementNode::new("body");
        let dom_tree = DomTree::new(root);

        // We can't easily test execute_typed without a real BrowserSession
        // but we can verify the structure is correct
        assert_eq!(dom_tree.count_interactive(), 0);
    }
}
