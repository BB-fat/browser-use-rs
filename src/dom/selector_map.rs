use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Information needed to locate an element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementSelector {
    /// CSS selector for the element
    pub css_selector: String,

    /// XPath selector (alternative to CSS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xpath: Option<String>,

    /// Element's tag name
    pub tag_name: String,

    /// Element's ID attribute (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Element's text content (truncated for display)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl ElementSelector {
    /// Create a new ElementSelector with CSS selector
    pub fn new(css_selector: impl Into<String>, tag_name: impl Into<String>) -> Self {
        Self {
            css_selector: css_selector.into(),
            xpath: None,
            tag_name: tag_name.into(),
            id: None,
            text: None,
        }
    }

    /// Builder method: set XPath
    pub fn with_xpath(mut self, xpath: impl Into<String>) -> Self {
        self.xpath = Some(xpath.into());
        self
    }

    /// Builder method: set ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Builder method: set text content
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Get the best selector to use (CSS preferred)
    pub fn best_selector(&self) -> &str {
        &self.css_selector
    }
}

/// Map of element indices to their selectors
/// Uses IndexMap to preserve insertion order
#[derive(Debug, Clone, Default)]
pub struct SelectorMap {
    /// Map from index to selector information
    map: IndexMap<usize, ElementSelector>,

    /// Next available index
    next_index: usize,
}

impl SelectorMap {
    /// Create a new empty SelectorMap
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            next_index: 0,
        }
    }

    /// Register a new element and return its assigned index
    pub fn register(&mut self, selector: ElementSelector) -> usize {
        let index = self.next_index;
        self.map.insert(index, selector);
        self.next_index += 1;
        index
    }

    /// Get selector by index
    pub fn get(&self, index: usize) -> Option<&ElementSelector> {
        self.map.get(&index)
    }

    /// Get mutable selector by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ElementSelector> {
        self.map.get_mut(&index)
    }

    /// Check if index exists
    pub fn contains(&self, index: usize) -> bool {
        self.map.contains_key(&index)
    }

    /// Remove an element by index
    pub fn remove(&mut self, index: usize) -> Option<ElementSelector> {
        self.map.shift_remove(&index)
    }

    /// Get the number of registered elements
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.map.clear();
        self.next_index = 0;
    }

    /// Iterate over all (index, selector) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&usize, &ElementSelector)> {
        self.map.iter()
    }

    /// Get all indices
    pub fn indices(&self) -> impl Iterator<Item = &usize> {
        self.map.keys()
    }

    /// Get all selectors
    pub fn selectors(&self) -> impl Iterator<Item = &ElementSelector> {
        self.map.values()
    }

    /// Find index by CSS selector
    pub fn find_by_css_selector(&self, css_selector: &str) -> Option<usize> {
        self.map
            .iter()
            .find(|(_, sel)| sel.css_selector == css_selector)
            .map(|(idx, _)| *idx)
    }

    /// Find index by element ID
    pub fn find_by_id(&self, id: &str) -> Option<usize> {
        self.map
            .iter()
            .find(|(_, sel)| sel.id.as_deref() == Some(id))
            .map(|(idx, _)| *idx)
    }

    /// Export to JSON for debugging
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_selector() {
        let selector = ElementSelector::new("#my-button", "button")
            .with_id("my-button")
            .with_text("Click me");

        assert_eq!(selector.css_selector, "#my-button");
        assert_eq!(selector.tag_name, "button");
        assert_eq!(selector.id, Some("my-button".to_string()));
        assert_eq!(selector.text, Some("Click me".to_string()));
        assert_eq!(selector.best_selector(), "#my-button");
    }

    #[test]
    fn test_selector_map_register() {
        let mut map = SelectorMap::new();

        let selector1 = ElementSelector::new("#btn1", "button");
        let selector2 = ElementSelector::new("#btn2", "button");

        let idx1 = map.register(selector1);
        let idx2 = map.register(selector2);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_selector_map_get() {
        let mut map = SelectorMap::new();

        let selector = ElementSelector::new("#test", "div").with_id("test");
        let index = map.register(selector);

        let retrieved = map.get(index).unwrap();
        assert_eq!(retrieved.css_selector, "#test");
        assert_eq!(retrieved.id, Some("test".to_string()));
    }

    #[test]
    fn test_selector_map_remove() {
        let mut map = SelectorMap::new();

        let selector = ElementSelector::new("#remove-me", "span");
        let index = map.register(selector);

        assert!(map.contains(index));

        let removed = map.remove(index);
        assert!(removed.is_some());
        assert!(!map.contains(index));
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_selector_map_clear() {
        let mut map = SelectorMap::new();

        map.register(ElementSelector::new("#one", "div"));
        map.register(ElementSelector::new("#two", "div"));
        map.register(ElementSelector::new("#three", "div"));

        assert_eq!(map.len(), 3);

        map.clear();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_selector_map_find() {
        let mut map = SelectorMap::new();

        let selector1 = ElementSelector::new("#btn1", "button").with_id("btn1");
        let selector2 = ElementSelector::new(".link", "a").with_id("link1");

        let idx1 = map.register(selector1);
        let _idx2 = map.register(selector2);

        let found = map.find_by_css_selector("#btn1");
        assert_eq!(found, Some(idx1));

        let found_by_id = map.find_by_id("link1");
        assert_eq!(found_by_id, Some(1));

        let not_found = map.find_by_css_selector("#nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_selector_map_iteration() {
        let mut map = SelectorMap::new();

        map.register(ElementSelector::new("#one", "div"));
        map.register(ElementSelector::new("#two", "div"));
        map.register(ElementSelector::new("#three", "div"));

        let indices: Vec<_> = map.indices().copied().collect();
        assert_eq!(indices, vec![0, 1, 2]);

        let css_selectors: Vec<_> = map.selectors().map(|s| s.css_selector.clone()).collect();
        assert_eq!(css_selectors, vec!["#one", "#two", "#three"]);
    }

    #[test]
    fn test_selector_serialization() {
        let selector = ElementSelector::new("#test", "button")
            .with_id("test")
            .with_text("Test Button");

        let json = serde_json::to_string(&selector).unwrap();
        let deserialized: ElementSelector = serde_json::from_str(&json).unwrap();

        assert_eq!(selector, deserialized);
    }

    #[test]
    fn test_selector_map_to_json() {
        let mut map = SelectorMap::new();

        map.register(ElementSelector::new("#btn", "button").with_text("Click"));
        map.register(ElementSelector::new("#link", "a").with_text("Visit"));

        let json = map.to_json().unwrap();
        assert!(json.contains("#btn"));
        assert!(json.contains("#link"));
        assert!(json.contains("Click"));
        assert!(json.contains("Visit"));
    }
}
