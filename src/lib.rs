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
