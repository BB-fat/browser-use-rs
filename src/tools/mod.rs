//! Browser automation tools module
//!
//! This module provides a framework for browser automation tools and
//! includes implementations of common browser operations.

pub mod navigate;
pub mod click;
pub mod input;
pub mod extract;
pub mod screenshot;
pub mod evaluate;
pub mod wait;

use crate::browser::BrowserSession;
use crate::dom::DomTree;
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Tool execution context
pub struct ToolContext<'a> {
    /// Browser session
    pub session: &'a BrowserSession,
    
    /// Optional DOM tree (extracted on demand)
    pub dom_tree: Option<DomTree>,
}

impl<'a> ToolContext<'a> {
    /// Create a new tool context
    pub fn new(session: &'a BrowserSession) -> Self {
        Self {
            session,
            dom_tree: None,
        }
    }

    /// Create a context with a pre-extracted DOM tree
    pub fn with_dom(session: &'a BrowserSession, dom_tree: DomTree) -> Self {
        Self {
            session,
            dom_tree: Some(dom_tree),
        }
    }

    /// Get or extract the DOM tree
    pub fn get_dom(&mut self) -> Result<&DomTree> {
        if self.dom_tree.is_none() {
            self.dom_tree = Some(self.session.extract_dom()?);
        }
        Ok(self.dom_tree.as_ref().unwrap())
    }
}

/// Result of tool execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution was successful
    pub success: bool,
    
    /// Result data (JSON value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    
    /// Error message if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(data: Option<Value>) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a successful result with data
    pub fn success_with<T: serde::Serialize>(data: T) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).ok(),
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failure result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Trait for browser automation tools
pub trait Tool: Send + Sync {
    /// Get tool name
    fn name(&self) -> &str;

    /// Get tool description
    fn description(&self) -> &str;

    /// Get tool parameter schema (JSON Schema)
    fn parameters_schema(&self) -> Value;

    /// Execute the tool
    fn execute(&self, params: Value, context: &mut ToolContext) -> Result<ToolResult>;
}

/// Tool registry for managing and accessing tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        
        // Register default tools
        registry.register(Arc::new(navigate::NavigateTool));
        registry.register(Arc::new(click::ClickTool));
        registry.register(Arc::new(input::InputTool));
        registry.register(Arc::new(extract::ExtractContentTool));
        registry.register(Arc::new(screenshot::ScreenshotTool));
        registry.register(Arc::new(evaluate::EvaluateTool));
        registry.register(Arc::new(wait::WaitTool));
        
        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Check if a tool exists
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// List all tool names
    pub fn list_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all tools
    pub fn all_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }

    /// Execute a tool by name
    pub fn execute(
        &self,
        name: &str,
        params: Value,
        context: &mut ToolContext,
    ) -> Result<ToolResult> {
        match self.get(name) {
            Some(tool) => tool.execute(params, context),
            None => Ok(ToolResult::failure(format!("Tool '{}' not found", name))),
        }
    }

    /// Get the number of registered tools
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success(Some(serde_json::json!({"url": "https://example.com"})));
        assert!(result.success);
        assert!(result.data.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_result_failure() {
        let result = ToolResult::failure("Test error");
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_tool_result_with_metadata() {
        let result = ToolResult::success(None)
            .with_metadata("duration_ms", serde_json::json!(100));
        
        assert!(result.metadata.contains_key("duration_ms"));
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::with_defaults();
        
        assert!(registry.has("navigate"));
        assert!(registry.has("click"));
        assert!(registry.has("input"));
        assert!(!registry.has("nonexistent"));
        
        assert!(registry.count() >= 7); // At least 7 default tools
    }

    #[test]
    fn test_tool_registry_list() {
        let registry = ToolRegistry::with_defaults();
        let names = registry.list_names();
        
        assert!(names.contains(&"navigate".to_string()));
        assert!(names.contains(&"click".to_string()));
    }
}
