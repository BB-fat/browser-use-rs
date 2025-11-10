//! # browser-use
//!
//! A Rust library for browser automation via Chrome DevTools Protocol (CDP), designed for AI agent integration.
//!
//! ## Features
//!
//! - **MCP Server**: Model Context Protocol server for AI-driven browser automation
//! - **Browser Session Management**: Launch or connect to Chrome/Chromium instances
//! - **Tool System**: High-level browser operations (navigate, click, input, extract, screenshot, etc.)
//! - **DOM Extraction**: Extract page structure with indexed interactive elements for AI-friendly targeting
//!
//! ## MCP Server
//!
//! The recommended way to use this library is via the Model Context Protocol (MCP) server,
//! which exposes browser automation tools to AI agents like Claude:
//!
//! ### Running the MCP Server
//!
//! ```bash
//! # Run headless browser
//! cargo run --bin mcp-server
//!
//! # Run with visible browser (useful for debugging)
//! cargo run --bin mcp-server -- --headed
//! ```
//!
//! ## Library Usage (Advanced)
//!
//! For direct integration in Rust applications:
//!
//! ### Basic Browser Automation
//!
//! ```rust,no_run
//! use browser_use::{BrowserSession, LaunchOptions};
//!
//! # fn main() -> browser_use::Result<()> {
//! // Launch a browser
//! let mut session = BrowserSession::launch(LaunchOptions::default())?;
//!
//! // Navigate to a page
//! session.navigate("https://example.com", None)?;
//!
//! // Extract DOM with indexed elements
//! let dom = session.extract_dom()?;
//! println!("Found {} interactive elements", dom.selector_map().len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Using the Tool System
//!
//! ```rust,no_run
//! use browser_use::{BrowserSession, LaunchOptions, ToolRegistry, ToolContext};
//! use serde_json::json;
//!
//! # fn main() -> browser_use::Result<()> {
//! let mut session = BrowserSession::launch(LaunchOptions::default())?;
//! let registry = ToolRegistry::new();
//! let mut context = ToolContext::new(&mut session);
//!
//! // Navigate using the tool system
//! registry.execute_tool("navigate", json!({"url": "https://example.com"}), &mut context)?;
//!
//! // Click an element by index
//! registry.execute_tool("click", json!({"index": 5}), &mut context)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### DOM Indexing for AI Agents
//!
//! The library automatically indexes interactive elements (buttons, links, inputs) with numeric IDs,
//! making it easier for AI agents to target elements without complex CSS selectors:
//!
//! ```rust,no_run
//! # use browser_use::{BrowserSession, LaunchOptions};
//! # fn main() -> browser_use::Result<()> {
//! # let mut session = BrowserSession::launch(LaunchOptions::default())?;
//! # session.navigate("https://example.com", None)?;
//! let dom = session.extract_dom()?;
//!
//! // Access elements by numeric index
//! if let Some(selector) = dom.selector_map().get_selector(5) {
//!     println!("Element 5 selector: {}", selector);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Overview
//!
//! - [`browser`]: Browser session management and configuration
//! - [`dom`]: DOM extraction, element indexing, and tree representation
//! - [`tools`]: Browser automation tools (navigate, click, input, extract, etc.)
//! - [`error`]: Error types and result aliases
//! - [`mcp`]: **Model Context Protocol server** (requires `mcp-server` feature) - **Start here for AI integration**

pub mod browser;
pub mod dom;
pub mod error;
pub mod tools;

#[cfg(feature = "mcp-server")]
pub mod mcp;

pub use browser::{BrowserSession, ConnectionOptions, LaunchOptions};
pub use dom::{BoundingBox, DomTree, ElementNode, ElementSelector, SelectorMap};
pub use error::{BrowserError, Result};
pub use tools::{Tool, ToolContext, ToolRegistry, ToolResult};

#[cfg(feature = "mcp-server")]
pub use mcp::BrowserServer;
