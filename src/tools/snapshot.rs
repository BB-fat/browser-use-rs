use crate::dom::ElementNode;
use crate::error::Result;
use crate::tools::{Tool, ToolContext, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for the snapshot tool (no parameters needed)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SnapshotParams {}

/// Tool for getting a snapshot of the page with indexed interactive elements
#[derive(Default)]
pub struct SnapshotTool;

impl Tool for SnapshotTool {
    type Params = SnapshotParams;

    fn name(&self) -> &str {
        "snapshot"
    }

    fn execute_typed(
        &self,
        _params: SnapshotParams,
        context: &mut ToolContext,
    ) -> Result<ToolResult> {
        // Get or extract the DOM tree
        let dom = context.get_dom()?;

        // Generate the snapshot by traversing the DOM tree
        let snapshot = generate_snapshot(&dom.root, 0);

        // Count interactive elements
        let interactive_count = dom.count_interactive();

        Ok(ToolResult::success_with(serde_json::json!({
            "snapshot": snapshot,
            "interactive_count": interactive_count
        })))
    }
}

/// Generate a Markdown-like snapshot of the page by traversing the DOM tree
fn generate_snapshot(node: &ElementNode, depth: usize) -> String {
    let mut output = String::new();

    // Skip invisible elements
    if !node.is_visible && depth > 0 {
        return output;
    }

    // Handle different element types
    match node.tag_name.as_str() {
        // Heading elements
        "h1" => {
            append_with_index(
                &mut output,
                node,
                &format!("# {}", get_text_content(node)),
                depth,
            );
        }
        "h2" => {
            append_with_index(
                &mut output,
                node,
                &format!("## {}", get_text_content(node)),
                depth,
            );
        }
        "h3" => {
            append_with_index(
                &mut output,
                node,
                &format!("### {}", get_text_content(node)),
                depth,
            );
        }
        "h4" => {
            append_with_index(
                &mut output,
                node,
                &format!("#### {}", get_text_content(node)),
                depth,
            );
        }
        "h5" => {
            append_with_index(
                &mut output,
                node,
                &format!("##### {}", get_text_content(node)),
                depth,
            );
        }
        "h6" => {
            append_with_index(
                &mut output,
                node,
                &format!("###### {}", get_text_content(node)),
                depth,
            );
        }
        // Button elements
        "button" => {
            let text = get_text_content(node);
            if !text.is_empty() {
                append_with_index(&mut output, node, &text, depth);
            } else {
                append_with_index(&mut output, node, "<button>", depth);
            }
        }
        // Link elements
        "a" => {
            let text = get_text_content(node);
            let href = node.get_attribute("href").map(|s| s.as_str()).unwrap_or("");
            if !text.is_empty() {
                if !href.is_empty() {
                    append_with_index(&mut output, node, &format!("{} ({})", text, href), depth);
                } else {
                    append_with_index(&mut output, node, &text, depth);
                }
            } else {
                append_with_index(&mut output, node, &format!("<link {}>", href), depth);
            }
        }
        // Input elements
        "input" => {
            let input_type = node
                .get_attribute("type")
                .map(|s| s.as_str())
                .unwrap_or("text");
            let placeholder = node
                .get_attribute("placeholder")
                .map(|s| format!(" placeholder=\"{}\"", s))
                .unwrap_or_default();
            append_with_index(
                &mut output,
                node,
                &format!("<input type=\"{}\"{}>,", input_type, placeholder),
                depth,
            );
        }
        // Textarea elements
        "textarea" => {
            let placeholder = node
                .get_attribute("placeholder")
                .map(|s| format!(" placeholder=\"{}\"", s))
                .unwrap_or_default();
            append_with_index(
                &mut output,
                node,
                &format!("<textarea{}>", placeholder),
                depth,
            );
        }
        // Select elements
        "select" => {
            append_with_index(&mut output, node, "<select>", depth);
        }
        // Label elements
        "label" => {
            let text = get_text_content(node);
            if !text.is_empty() {
                append_with_index(&mut output, node, &text, depth);
            }
        }
        // Paragraph and div elements with text
        "p" | "div" | "span" | "section" | "article" | "main" | "header" | "footer" | "nav" => {
            let text = get_direct_text_content(node);
            if !text.is_empty() {
                append_with_index(&mut output, node, &text, depth);
            }
            // Process children
            for child in &node.children {
                let child_output = generate_snapshot(child, depth + 1);
                if !child_output.is_empty() {
                    output.push_str(&child_output);
                }
            }
            return output;
        }
        // List items
        "li" => {
            let text = get_direct_text_content(node);
            let indent = "  ".repeat(depth.saturating_sub(1));
            if !text.is_empty() {
                append_with_index(&mut output, node, &format!("{}• {}", indent, text), depth);
            } else {
                append_with_index(&mut output, node, &format!("{}• ", indent), depth);
            }
            // Process children
            for child in &node.children {
                let child_output = generate_snapshot(child, depth + 1);
                if !child_output.is_empty() {
                    output.push_str(&child_output);
                }
            }
            return output;
        }
        // Container elements - just process children
        "body" | "ul" | "ol" | "form" | "fieldset" | "table" | "tbody" | "thead" | "tr" => {
            for child in &node.children {
                let child_output = generate_snapshot(child, depth + 1);
                if !child_output.is_empty() {
                    output.push_str(&child_output);
                }
            }
            return output;
        }
        // Other interactive elements with role attributes
        _ => {
            if node.is_interactive {
                let text = get_text_content(node);
                if !text.is_empty() {
                    append_with_index(&mut output, node, &text, depth);
                } else {
                    append_with_index(&mut output, node, &format!("<{}>", node.tag_name), depth);
                }
            } else {
                // Non-interactive elements with text
                let text = get_direct_text_content(node);
                if !text.is_empty() {
                    output.push_str(&text);
                    output.push('\n');
                }
                // Process children
                for child in &node.children {
                    let child_output = generate_snapshot(child, depth + 1);
                    if !child_output.is_empty() {
                        output.push_str(&child_output);
                    }
                }
            }
            return output;
        }
    }

    // Process children for elements that haven't returned yet
    for child in &node.children {
        let child_output = generate_snapshot(child, depth + 1);
        if !child_output.is_empty() {
            output.push_str(&child_output);
        }
    }

    output
}

/// Append content with index marker if the element is interactive
fn append_with_index(output: &mut String, node: &ElementNode, content: &str, _depth: usize) {
    if let Some(index) = node.index {
        output.push_str(&format!("[{}] {}\n", index, content));
    } else {
        output.push_str(content);
        output.push('\n');
    }
}

/// Get the text content of an element (including all descendants)
fn get_text_content(node: &ElementNode) -> String {
    if let Some(text) = &node.text_content {
        let trimmed = text.trim();
        if trimmed.len() > 200 {
            format!("{}...", &trimmed[..197])
        } else {
            trimmed.to_string()
        }
    } else {
        String::new()
    }
}

/// Get only the direct text content of an element (not including children)
fn get_direct_text_content(node: &ElementNode) -> String {
    if node.children.is_empty() {
        get_text_content(node)
    } else {
        // If has children, only use text_content if it's short (likely direct text)
        if let Some(text) = &node.text_content {
            let trimmed = text.trim();
            if !trimmed.is_empty()
                && trimmed.len() < 100
                && !node.children.iter().any(|c| c.is_visible)
            {
                return trimmed.to_string();
            }
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::DomTree;
    use crate::dom::ElementNode;

    #[test]
    fn test_empty_dom_tree() {
        let root = ElementNode::new("body");
        let dom_tree = DomTree::new(root);
        assert_eq!(dom_tree.count_interactive(), 0);
    }

    #[test]
    fn test_generate_snapshot_simple() {
        let mut root = ElementNode::new("body");
        root.is_visible = true;

        let mut heading = ElementNode::new("h1");
        heading.text_content = Some("Welcome".to_string());
        heading.is_visible = true;
        root.add_child(heading);

        let mut button = ElementNode::new("button");
        button.text_content = Some("Click me".to_string());
        button.is_visible = true;
        button.is_interactive = true;
        button.index = Some(0);
        root.add_child(button);

        let snapshot = generate_snapshot(&root, 0);
        assert!(snapshot.contains("# Welcome"));
        assert!(snapshot.contains("[0] Click me"));
    }

    #[test]
    fn test_generate_snapshot_with_links() {
        let mut root = ElementNode::new("body");
        root.is_visible = true;

        let mut link = ElementNode::new("a");
        link.add_attribute("href", "https://example.com");
        link.text_content = Some("Example Link".to_string());
        link.is_visible = true;
        link.is_interactive = true;
        link.index = Some(5);
        root.add_child(link);

        let snapshot = generate_snapshot(&root, 0);
        assert!(snapshot.contains("[5] Example Link (https://example.com)"));
    }

    #[test]
    fn test_generate_snapshot_with_input() {
        let mut root = ElementNode::new("body");
        root.is_visible = true;

        let mut input = ElementNode::new("input");
        input.add_attribute("type", "text");
        input.add_attribute("placeholder", "Enter your name");
        input.is_visible = true;
        input.is_interactive = true;
        input.index = Some(10);
        root.add_child(input);

        let snapshot = generate_snapshot(&root, 0);
        assert!(snapshot.contains("[10]"));
        assert!(snapshot.contains("input"));
        assert!(snapshot.contains("placeholder=\"Enter your name\""));
    }
}
