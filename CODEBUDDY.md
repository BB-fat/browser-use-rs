# CODEBUDDY.md This file provides guidance to CodeBuddy Code when working with code in this repository.

## Project Overview

This is a Rust implementation of a browser automation MCP (Model Context Protocol) server, inspired by the TypeScript implementation in `example/browser/src`. The project provides browser automation capabilities through both a standalone MCP server binary and a reusable library.

### Key Dependencies
- **headless_chrome**: Rust library for controlling Chrome/Chromium browsers via CDP (Chrome DevTools Protocol)
- **rmcp**: Rust MCP (Model Context Protocol) server implementation framework

### Reference Implementation
The `example/browser/` directory contains the TypeScript reference implementation from `@agent-infra/mcp-server-browser`. This serves as the specification for tool behavior and capabilities.

## Commands

### Build
```bash
# Build library and binary
cargo build

# Build release version
cargo build --release

# Build library only
cargo build --lib

# Build binary only
cargo build --bin browser-use
```

### Run
```bash
# Run MCP server in stdio mode
cargo run

# Run with configuration
cargo run -- --headless --viewport-size "1920,1080"

# Run as HTTP server
cargo run -- --port 8089
```

### Test
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test tools::action::tests

# Run integration tests
cargo test --test '*'

# Run tests for library only
cargo test --lib
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate docs for library API
cargo doc --no-deps --lib
```

### Check/Lint
```bash
# Fast compilation check
cargo check

# Linting with clippy
cargo clippy

# Format code
cargo fmt
```

## Architecture

### Design Principles (Rust-Specific)

1. **Library-First Design**: Core functionality exposed as a library (`src/lib.rs`), with binary as a thin wrapper
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees (no runtime schema validation where possible)
3. **Ownership Model**: Use Rust ownership for resource management (browser instances, tabs, downloads)
4. **Error Handling**: Use `Result<T, E>` throughout, no panics in library code
5. **Async Runtime**: Use `tokio` for async operations (browser interactions are inherently async)
6. **Zero-Cost Abstractions**: Avoid runtime overhead through generic programming and trait objects where appropriate

### Project Structure (Rust-Idiomatic)

```
browser-use/
├── src/
│   ├── lib.rs                 # Library root, public API exports
│   ├── main.rs                # Binary entry point (CLI)
│   │
│   ├── browser/               # Browser lifecycle management
│   │   ├── mod.rs             # Browser initialization, connection
│   │   ├── session.rs         # BrowserSession (owns Browser + active Tab)
│   │   └── config.rs          # LaunchOptions, ConnectionOptions
│   │
│   ├── dom/                   # DOM tree and element selection
│   │   ├── mod.rs
│   │   ├── tree.rs            # DOM tree building (inject JS, parse)
│   │   ├── element.rs         # ElementNode, ElementIndex
│   │   └── selector_map.rs    # IndexMap<usize, ElementNode>
│   │
│   ├── tools/                 # Tool implementations
│   │   ├── mod.rs             # Tool trait definition, registration
│   │   ├── navigate.rs        # Navigation tools
│   │   ├── content.rs         # Content extraction tools
│   │   ├── action.rs          # Interactive action tools
│   │   ├── tabs.rs            # Tab management tools
│   │   ├── evaluate.rs        # JavaScript evaluation
│   │   ├── vision.rs          # Vision mode tools (feature-gated)
│   │   └── download.rs        # Download management
│   │
│   ├── mcp/                   # MCP server integration
│   │   ├── mod.rs
│   │   ├── server.rs          # McpServer wrapper
│   │   ├── schema.rs          # Tool schema definitions (serde)
│   │   └── transport.rs       # Transport layer (stdio, HTTP, SSE)
│   │
│   ├── resources/             # MCP resources
│   │   ├── mod.rs
│   │   ├── console.rs         # console://logs
│   │   ├── screenshot.rs      # screenshot://{name}
│   │   └── download.rs        # download://{name}
│   │
│   └── error.rs               # Error types (thiserror)
│
├── tests/                     # Integration tests
│   ├── tools_navigation.rs
│   ├── tools_content.rs
│   ├── tools_action.rs
│   └── browser_lifecycle.rs
│
└── examples/                  # Library usage examples
    ├── standalone_automation.rs
    └── custom_mcp_server.rs
```

### Core Types and Traits

#### BrowserSession (owns browser state)
```rust
pub struct BrowserSession {
    browser: Arc<Browser>,
    active_tab: Arc<RwLock<Tab>>,
    selector_map: Arc<RwLock<IndexMap<usize, ElementNode>>>,
    downloads: Arc<RwLock<Vec<Download>>>,
    screenshots: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

**Design rationale**:
- `Arc` for shared ownership across async tasks
- `RwLock` for interior mutability (multiple readers, single writer)
- Encapsulates all browser-related state (no global singletons)
- Drop implementation ensures browser cleanup

#### Tool Trait (Rust async trait)
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> ToolSchema;
    
    async fn execute(
        &self,
        session: &BrowserSession,
        args: Value,
    ) -> Result<ToolResult>;
}
```

**Design rationale**:
- Trait objects enable dynamic tool registration
- `Send + Sync` bounds for multi-threaded async runtime
- `ToolSchema` uses serde for JSON schema generation
- Errors propagated via `Result` (no exceptions)

#### Error Handling (thiserror)
```rust
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Element not found: index {0}")]
    ElementNotFound(usize),
    
    #[error("Navigation timeout: {0}")]
    NavigationTimeout(String),
    
    #[error("CDP protocol error: {0}")]
    CdpError(#[from] cdp::Error),
    
    // ... more variants
}

pub type Result<T> = std::result::Result<T, BrowserError>;
```

### Complete Tool List (Must Implement All)

#### Navigation Tools
1. **browser_navigate**: Navigate to URL
2. **browser_go_back**: Go to previous page
3. **browser_go_forward**: Go to next page

#### Content Tools
4. **browser_get_markdown**: Extract markdown from current page
5. **browser_get_text**: Get text content from current page
6. **browser_read_links**: Get all links on page

#### Action Tools
7. **browser_click**: Click element by index
8. **browser_hover**: Hover element by index or selector
9. **browser_select**: Select dropdown option by index or selector
10. **browser_form_input_fill**: Fill form input by index or selector
11. **browser_scroll**: Scroll page by pixel amount
12. **browser_press_key**: Press keyboard key
13. **browser_get_clickable_elements**: Get clickable/hoverable/selectable elements with indices
14. **browser_screenshot**: Take screenshot of page or element

#### Tab Tools
15. **browser_new_tab**: Open new tab with URL
16. **browser_tab_list**: List all tabs
17. **browser_switch_tab**: Switch to tab by index
18. **browser_close_tab**: Close current tab

#### Evaluate Tools
19. **browser_evaluate**: Execute JavaScript in console

#### Vision Tools (feature = "vision")
20. **browser_vision_screen_capture**: Take screenshot for vision mode
21. **browser_vision_screen_click**: Click based on vision coordinates

#### Download Tools
22. **browser_get_download_list**: Get list of downloaded files

#### Browser Management
23. **browser_close**: Close browser

### Key Implementation Details

#### Element Selection System
The reference implementation uses **label indices** instead of pixel coordinates:
1. `buildDomTree()` creates accessibility tree with highlighted indices
2. Elements get numeric labels (0, 1, 2, ...)
3. Tools use these indices to reference elements
4. `selector_map` (in Rust: `IndexMap<usize, ElementNode>`) maps indices to element data
5. `locate_element()` uses element data to re-find element in DOM

#### DOM Tree Building
- JavaScript injected into page context (ship as static string in binary)
- Creates accessibility-based element tree
- Highlights interactive elements with numeric overlays
- Returns JSON tree with element metadata (tag, attributes, aria labels, bbox)
- Parsed in Rust using `serde_json`

#### Configuration Options (CLI arguments)
Use `clap` for CLI parsing:
- `--browser`: chrome/edge/firefox
- `--cdp-endpoint`, `--ws-endpoint`: remote browser connection
- `--executable-path`: custom browser binary
- `--headless`: run in headless mode
- `--port`: enable HTTP transport (remote MCP server)
- `--vision`: enable vision mode tools (feature flag)
- `--viewport-size`: "width,height"
- `--user-agent`: custom UA string
- `--user-data-dir`: persistent browser profile
- `--output-dir`: directory for downloads/screenshots
- `--proxy-server`, `--proxy-bypass`: proxy configuration

#### Transport Modes
1. **Stdio** (default): Standard MCP stdio transport
2. **HTTP** (`--port`): HTTP server with MCP endpoints

#### Resources
MCP resources expose runtime artifacts:
- `console://logs`: Browser console logs
- `screenshot://{name}`: Stored screenshots
- `download://{name}`: Downloaded files

### Library API Design

#### Public API (src/lib.rs exports)
```rust
// Core types
pub use browser::BrowserSession;
pub use browser::config::{LaunchOptions, ConnectionOptions};
pub use error::{BrowserError, Result};

// Tool system
pub use tools::{Tool, ToolRegistry, ToolResult};

// DOM types
pub use dom::{ElementNode, DomTree};

// MCP server (optional)
#[cfg(feature = "mcp-server")]
pub use mcp::McpServer;

// Builder pattern for ergonomic API
pub struct BrowserSessionBuilder { /* ... */ }
```

#### Example Library Usage
```rust
use browser_use::{BrowserSession, LaunchOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create browser session
    let session = BrowserSession::builder()
        .headless(true)
        .viewport(1920, 1080)
        .launch()
        .await?;
    
    // Navigate to page
    session.navigate("https://example.com").await?;
    
    // Get clickable elements
    let elements = session.get_clickable_elements().await?;
    println!("Found {} clickable elements", elements.len());
    
    // Click first element
    session.click_element(0).await?;
    
    // Extract text
    let text = session.get_text().await?;
    
    Ok(())
}
```

### Testing Strategy

#### Unit Tests (in module files)
- Test pure functions (DOM tree parsing, element location logic)
- Mock browser responses where possible
- Fast, no actual browser required

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_element_node() {
        let json = r#"{"tag": "button", "index": 5}"#;
        let node = parse_element_node(json).unwrap();
        assert_eq!(node.index, 5);
    }
}
```

#### Integration Tests (tests/ directory)
- Launch actual browser (use `--headless` for CI)
- Test tool behavior end-to-end
- Use local test HTML files to ensure deterministic results

Example:
```rust
#[tokio::test]
async fn test_click_element() {
    let session = BrowserSession::builder()
        .headless(true)
        .launch()
        .await
        .unwrap();
    
    // Navigate to test page
    let test_html = format!("file://{}/tests/fixtures/buttons.html", env!("CARGO_MANIFEST_DIR"));
    session.navigate(&test_html).await.unwrap();
    
    // Get clickable elements
    let elements = session.get_clickable_elements().await.unwrap();
    assert!(elements.len() > 0);
    
    // Click first button
    let result = session.click_element(0).await;
    assert!(result.is_ok());
}
```

#### Critical Tools Requiring Tests
- **browser_click**: Element location, click handling, download detection
- **browser_form_input_fill**: Input clearing, typing, validation
- **browser_get_clickable_elements**: DOM tree building, element indexing
- **browser_navigate**: Timeout handling, error cases
- **browser_screenshot**: Full page vs element screenshots
- **browser_evaluate**: JavaScript execution, result serialization

### Cargo Features

```toml
[features]
default = ["mcp-server"]
mcp-server = ["rmcp", "tokio"]
vision = []  # Enable vision mode tools
```

Usage:
- `cargo build --no-default-features --lib`: Build core library only
- `cargo build --features vision`: Include vision tools
- `cargo build --bin browser-use`: Build MCP server binary

## Important Implementation Notes

- **No global state in library code**: `BrowserSession` owns all state (enables multiple concurrent sessions)
- **Tool parity is critical**: All 23 tools must match reference behavior
- **Element indexing**: Label-based selection system (not pixel-based)
- **Error handling**: Graceful timeout handling (navigation timeouts are warnings, not errors)
- **Resource cleanup**: Implement `Drop` for `BrowserSession` to close browser
- **Async everywhere**: All browser operations are async (use `tokio` runtime)
- **Download tracking**: Use CDP `Page.downloadWillBegin` and `Page.downloadProgress` events
- **Popup handling**: Listen for `Target.targetCreated` events, switch to new tabs automatically
- **Thread safety**: All shared state uses `Arc<RwLock<T>>` for safe concurrent access
