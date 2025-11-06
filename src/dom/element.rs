use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a DOM element node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementNode {
    /// HTML tag name (e.g., "div", "button", "input")
    pub tag_name: String,

    /// Element attributes (e.g., id, class, href, etc.)
    #[serde(default)]
    pub attributes: HashMap<String, String>,

    /// Text content of the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,

    /// Child elements
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ElementNode>,

    /// Index assigned to this element (for interactive elements)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// Whether the element is visible in the viewport
    #[serde(default)]
    pub is_visible: bool,

    /// Whether the element is interactive (clickable, input, etc.)
    #[serde(default)]
    pub is_interactive: bool,

    /// Bounding box information (x, y, width, height)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
}

/// Bounding box coordinates for an element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ElementNode {
    /// Create a new ElementNode
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: HashMap::new(),
            text_content: None,
            children: Vec::new(),
            index: None,
            is_visible: false,
            is_interactive: false,
            bounding_box: None,
        }
    }

    /// Builder method: set attributes
    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Builder method: set text content
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_content = Some(text.into());
        self
    }

    /// Builder method: set children
    pub fn with_children(mut self, children: Vec<ElementNode>) -> Self {
        self.children = children;
        self
    }

    /// Builder method: set index
    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Builder method: set visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.is_visible = visible;
        self
    }

    /// Builder method: set interactivity
    pub fn with_interactivity(mut self, interactive: bool) -> Self {
        self.is_interactive = interactive;
        self
    }

    /// Builder method: set bounding box
    pub fn with_bounding_box(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.bounding_box = Some(BoundingBox {
            x,
            y,
            width,
            height,
        });
        self
    }

    /// Add a single attribute
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Add a child element
    pub fn add_child(&mut self, child: ElementNode) {
        self.children.push(child);
    }

    /// Get attribute value by key
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Check if element has a specific class
    pub fn has_class(&self, class_name: &str) -> bool {
        if let Some(classes) = self.attributes.get("class") {
            classes.split_whitespace().any(|c| c == class_name)
        } else {
            false
        }
    }

    /// Get element ID
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    /// Check if element is a specific tag
    pub fn is_tag(&self, tag: &str) -> bool {
        self.tag_name.eq_ignore_ascii_case(tag)
    }

    /// Determine if this element should be considered interactive
    pub fn compute_interactivity(&mut self) {
        // Interactive tags
        let interactive_tags = ["button", "a", "input", "select", "textarea", "label"];

        // Check if tag is interactive
        let tag_is_interactive = interactive_tags.iter().any(|&tag| self.is_tag(tag));

        // Check for onclick or other event handlers
        let has_event_handler = self.attributes.keys().any(|k| {
            k.starts_with("on")
                || k == "role" && self.attributes.get("role").map_or(false, |r| r == "button")
        });

        // Check for clickable role
        let has_clickable_role = self.get_attribute("role").map_or(false, |r| {
            ["button", "link", "tab", "menuitem"].contains(&r.as_str())
        });

        self.is_interactive = tag_is_interactive || has_event_handler || has_clickable_role;
    }

    /// Simplify element by removing unnecessary children (like scripts, styles)
    pub fn simplify(&mut self) {
        // Remove script, style, and noscript elements
        self.children
            .retain(|child| !matches!(child.tag_name.as_str(), "script" | "style" | "noscript"));

        // Recursively simplify children
        for child in &mut self.children {
            child.simplify();
        }
    }

    /// Convert to a simplified string representation
    pub fn to_simple_string(&self) -> String {
        let mut parts = vec![format!("<{}", self.tag_name)];

        if let Some(id) = self.id() {
            parts.push(format!(" id=\"{}\"", id));
        }

        if let Some(class) = self.attributes.get("class") {
            parts.push(format!(" class=\"{}\"", class));
        }

        if let Some(index) = self.index {
            parts.push(format!(" data-index=\"{}\"", index));
        }

        parts.push(">".to_string());

        if let Some(text) = &self.text_content {
            if !text.trim().is_empty() {
                parts.push(text.trim().to_string());
            }
        }

        parts.join("")
    }
}

impl BoundingBox {
    /// Create a new BoundingBox
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if the bounding box is visible (has non-zero dimensions)
    pub fn is_visible(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }

    /// Calculate the area of the bounding box
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_node_creation() {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), "test-id".to_string());
        attrs.insert("class".to_string(), "btn primary".to_string());

        let element = ElementNode::new("button")
            .with_attributes(attrs)
            .with_text("Click me")
            .with_index(1)
            .with_visibility(true)
            .with_interactivity(true);

        assert_eq!(element.tag_name, "button");
        assert_eq!(element.id(), Some(&"test-id".to_string()));
        assert_eq!(element.text_content, Some("Click me".to_string()));
        assert_eq!(element.index, Some(1));
        assert!(element.is_visible);
        assert!(element.is_interactive);
    }

    #[test]
    fn test_has_class() {
        let mut element = ElementNode::new("div");
        element.add_attribute("class", "container main active");

        assert!(element.has_class("container"));
        assert!(element.has_class("main"));
        assert!(element.has_class("active"));
        assert!(!element.has_class("hidden"));
    }

    #[test]
    fn test_compute_interactivity() {
        let mut button = ElementNode::new("button");
        button.compute_interactivity();
        assert!(button.is_interactive);

        let mut div = ElementNode::new("div");
        div.compute_interactivity();
        assert!(!div.is_interactive);

        let mut clickable_div = ElementNode::new("div");
        clickable_div.add_attribute("onclick", "alert('hi')");
        clickable_div.compute_interactivity();
        assert!(clickable_div.is_interactive);

        let mut role_button = ElementNode::new("div");
        role_button.add_attribute("role", "button");
        role_button.compute_interactivity();
        assert!(role_button.is_interactive);
    }

    #[test]
    fn test_simplify() {
        let mut parent = ElementNode::new("div");
        parent.add_child(ElementNode::new("p").with_text("Content"));
        parent.add_child(ElementNode::new("script").with_text("alert('test')"));
        parent.add_child(ElementNode::new("style").with_text(".test { color: red; }"));
        parent.add_child(ElementNode::new("span").with_text("More content"));

        parent.simplify();

        assert_eq!(parent.children.len(), 2);
        assert!(parent.children[0].is_tag("p"));
        assert!(parent.children[1].is_tag("span"));
    }

    #[test]
    fn test_serialization() {
        let element = ElementNode::new("button")
            .with_text("Click")
            .with_index(5)
            .with_visibility(true);

        let json = serde_json::to_string(&element).unwrap();
        let deserialized: ElementNode = serde_json::from_str(&json).unwrap();

        assert_eq!(element, deserialized);
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);

        assert!(bbox.is_visible());
        assert_eq!(bbox.area(), 5000.0);

        let invisible_bbox = BoundingBox::new(0.0, 0.0, 0.0, 0.0);
        assert!(!invisible_bbox.is_visible());
    }

    #[test]
    fn test_to_simple_string() {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), "my-btn".to_string());
        attrs.insert("class".to_string(), "btn primary".to_string());

        let element = ElementNode::new("button")
            .with_attributes(attrs)
            .with_text("Submit")
            .with_index(10);

        let simple = element.to_simple_string();
        assert!(simple.contains("<button"));
        assert!(simple.contains("id=\"my-btn\""));
        assert!(simple.contains("class=\"btn primary\""));
        assert!(simple.contains("data-index=\"10\""));
        assert!(simple.contains("Submit"));
    }
}
