use crate::dom::element::ElementNode;
use crate::dom::selector_map::{ElementSelector, SelectorMap};
use crate::error::{BrowserError, Result};
use headless_chrome::Tab;
use std::sync::Arc;

/// Represents the DOM tree of a web page
#[derive(Debug, Clone)]
pub struct DomTree {
    /// Root element of the DOM tree
    pub root: ElementNode,

    /// Map of indices to element selectors
    pub selector_map: SelectorMap,
}

impl DomTree {
    /// Create a new empty DomTree
    pub fn new(root: ElementNode) -> Self {
        Self {
            root,
            selector_map: SelectorMap::new(),
        }
    }

    /// Build DOM tree from a browser tab
    pub fn from_tab(tab: &Arc<Tab>) -> Result<Self> {
        // JavaScript code to extract simplified DOM structure
        // This returns a JSON string
        let js_code = include_str!("extract_dom.js");

        // Execute JavaScript to extract DOM
        let result = tab.evaluate(js_code, false).map_err(|e| {
            BrowserError::DomParseFailed(format!("Failed to execute DOM extraction script: {}", e))
        })?;

        // Get the JSON string value
        let json_value = result.value.ok_or_else(|| {
            BrowserError::DomParseFailed("No value returned from DOM extraction".to_string())
        })?;

        // The JavaScript returns a JSON string, so we need to parse it as a string first
        let json_str: String = serde_json::from_value(json_value).map_err(|e| {
            BrowserError::DomParseFailed(format!("Failed to get JSON string: {}", e))
        })?;

        // Then parse the JSON string into ElementNode
        let root: ElementNode = serde_json::from_str(&json_str).map_err(|e| {
            BrowserError::DomParseFailed(format!("Failed to parse DOM JSON: {}", e))
        })?;

        let mut tree = Self::new(root);
        tree.build_selector_map();

        Ok(tree)
    }

    /// Build the selector map by traversing the DOM tree
    fn build_selector_map(&mut self) {
        self.selector_map.clear();
        let mut index_counter = 0;
        Self::traverse_and_index_static(
            &mut self.root,
            "body",
            &mut self.selector_map,
            &mut index_counter,
        );
    }

    /// Static method to recursively traverse and index elements
    fn traverse_and_index_static(
        node: &mut ElementNode,
        css_path: &str,
        selector_map: &mut SelectorMap,
        _index_counter: &mut usize,
    ) {
        // Compute interactivity for this node
        node.compute_interactivity();

        // If the element is interactive, assign it an index
        if node.is_interactive && node.is_visible {
            let selector = Self::build_selector_static(node, css_path);
            let index = selector_map.register(selector);
            node.index = Some(index);
        }

        // Recursively process children
        for (i, child) in node.children.iter_mut().enumerate() {
            let child_path = format!("{} > {}:nth-child({})", css_path, child.tag_name, i + 1);
            Self::traverse_and_index_static(child, &child_path, selector_map, _index_counter);
        }
    }

    /// Build an ElementSelector for a given node (static version)
    fn build_selector_static(node: &ElementNode, css_path: &str) -> ElementSelector {
        // Prefer ID selector if available
        let css_selector = if let Some(id) = &node.id() {
            format!("#{}", id)
        } else if let Some(class) = node.get_attribute("class") {
            format!(
                "{}.{}",
                node.tag_name,
                class.split_whitespace().next().unwrap_or("")
            )
        } else {
            css_path.to_string()
        };

        let mut selector = ElementSelector::new(css_selector, &node.tag_name);

        if let Some(id) = node.id() {
            selector = selector.with_id(id);
        }

        if let Some(text) = &node.text_content {
            // Truncate text for display
            let truncated = if text.len() > 50 {
                format!("{}...", &text[..47])
            } else {
                text.clone()
            };
            selector = selector.with_text(truncated);
        }

        selector
    }

    /// Simplify the DOM tree by removing unnecessary elements
    pub fn simplify(&mut self) {
        self.root.simplify();
        self.build_selector_map(); // Rebuild map after simplification
    }

    /// Convert the DOM tree to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.root).map_err(|e| {
            BrowserError::DomParseFailed(format!("Failed to serialize DOM to JSON: {}", e))
        })
    }

    /// Get element selector by index
    pub fn get_selector(&self, index: usize) -> Option<&ElementSelector> {
        self.selector_map.get(index)
    }

    /// Get all interactive element indices
    pub fn interactive_indices(&self) -> Vec<usize> {
        self.selector_map.indices().copied().collect()
    }

    /// Count total elements in the tree
    pub fn count_elements(&self) -> usize {
        self.count_elements_recursive(&self.root)
    }

    fn count_elements_recursive(&self, node: &ElementNode) -> usize {
        1 + node
            .children
            .iter()
            .map(|c| self.count_elements_recursive(c))
            .sum::<usize>()
    }

    /// Count interactive elements
    pub fn count_interactive(&self) -> usize {
        self.selector_map.len()
    }

    /// Find element node by index (traverse the tree)
    pub fn find_node_by_index(&self, index: usize) -> Option<&ElementNode> {
        self.find_node_by_index_recursive(&self.root, index)
    }

    fn find_node_by_index_recursive<'a>(
        &self,
        node: &'a ElementNode,
        target_index: usize,
    ) -> Option<&'a ElementNode> {
        if node.index == Some(target_index) {
            return Some(node);
        }

        for child in &node.children {
            if let Some(found) = self.find_node_by_index_recursive(child, target_index) {
                return Some(found);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> ElementNode {
        let mut root = ElementNode::new("body");

        let mut header = ElementNode::new("header");
        let mut nav_button = ElementNode::new("button");
        nav_button.add_attribute("id", "nav-btn");
        nav_button.text_content = Some("Menu".to_string());
        nav_button.is_visible = true;
        header.add_child(nav_button);

        let mut main = ElementNode::new("main");
        let mut link = ElementNode::new("a");
        link.add_attribute("href", "/page");
        link.text_content = Some("Click here".to_string());
        link.is_visible = true;
        main.add_child(link);

        let mut div = ElementNode::new("div");
        div.add_attribute("class", "content");
        div.text_content = Some("Some text".to_string());
        main.add_child(div);

        root.add_child(header);
        root.add_child(main);

        root
    }

    #[test]
    fn test_dom_tree_creation() {
        let root = create_test_tree();
        let tree = DomTree::new(root);

        assert_eq!(tree.root.tag_name, "body");
        assert_eq!(tree.root.children.len(), 2);
    }

    #[test]
    fn test_build_selector_map() {
        let root = create_test_tree();
        let mut tree = DomTree::new(root);
        tree.build_selector_map();

        // Should have 2 interactive elements: button and link
        assert_eq!(tree.count_interactive(), 2);
    }

    #[test]
    fn test_find_node_by_index() {
        let root = create_test_tree();
        let mut tree = DomTree::new(root);
        tree.build_selector_map();

        let indices = tree.interactive_indices();
        assert!(!indices.is_empty());

        for &index in &indices {
            let node = tree.find_node_by_index(index);
            assert!(node.is_some());
            assert_eq!(node.unwrap().index, Some(index));
        }
    }

    #[test]
    fn test_count_elements() {
        let root = create_test_tree();
        let tree = DomTree::new(root);

        // body > header > button, body > main > link, div
        // Total: body(1) + header(1) + button(1) + main(1) + link(1) + div(1) = 6
        assert_eq!(tree.count_elements(), 6);
    }

    #[test]
    fn test_simplify() {
        let mut root = ElementNode::new("body");
        root.add_child(ElementNode::new("p").with_text("Content"));
        root.add_child(ElementNode::new("script").with_text("alert('test')"));
        root.add_child(ElementNode::new("style").with_text(".test {}"));

        let mut tree = DomTree::new(root);
        tree.simplify();

        assert_eq!(tree.root.children.len(), 1);
        assert!(tree.root.children[0].is_tag("p"));
    }

    #[test]
    fn test_to_json() {
        let mut root = ElementNode::new("div");
        root.add_attribute("id", "container");
        root.add_child(ElementNode::new("span").with_text("Hello"));

        let tree = DomTree::new(root);
        let json = tree.to_json().unwrap();

        assert!(json.contains("\"tag_name\": \"div\""));
        assert!(json.contains("\"id\": \"container\""));
        assert!(json.contains("\"span\""));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_get_selector() {
        let root = create_test_tree();
        let mut tree = DomTree::new(root);
        tree.build_selector_map();

        let indices = tree.interactive_indices();
        for &index in &indices {
            let selector = tree.get_selector(index);
            assert!(selector.is_some());
        }
    }
}
