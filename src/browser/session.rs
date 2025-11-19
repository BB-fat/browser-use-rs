use crate::browser::config::{ConnectionOptions, LaunchOptions};
use crate::dom::DomTree;
use crate::error::{BrowserError, Result};
use crate::tools::{ToolContext, ToolRegistry};
use headless_chrome::{Browser, Tab};
use std::sync::Arc;
use std::time::Duration;

/// Wrapper for Tab and Element to maintain proper lifetime relationships
pub struct TabElement<'a> {
    pub tab: Arc<Tab>,
    pub element: headless_chrome::Element<'a>,
}

/// Browser session that manages a Chrome/Chromium instance
pub struct BrowserSession {
    /// The underlying headless_chrome Browser instance
    browser: Browser,

    /// Tool registry for executing browser automation tools
    tool_registry: ToolRegistry,
}

impl BrowserSession {
    /// Launch a new browser instance with the given options
    pub fn launch(options: LaunchOptions) -> Result<Self> {
        let mut launch_opts = headless_chrome::LaunchOptions::default();

        // Set the browser's idle timeout to 1 hour (default is 30 seconds) to prevent the session from closing too soon
        launch_opts.idle_browser_timeout = Duration::from_secs(60 * 60);

        // Configure headless mode
        launch_opts.headless = options.headless;

        // Set window size
        launch_opts.window_size = Some((options.window_width, options.window_height));

        // Set Chrome binary path if provided
        if let Some(path) = options.chrome_path {
            launch_opts.path = Some(path);
        }

        // Set user data directory if provided
        if let Some(dir) = options.user_data_dir {
            launch_opts.user_data_dir = Some(dir);
        }

        // Set sandbox mode
        launch_opts.sandbox = options.sandbox;

        // Launch browser
        let browser =
            Browser::new(launch_opts).map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        browser
            .new_tab()
            .map_err(|e| BrowserError::LaunchFailed(format!("Failed to create tab: {}", e)))?;

        Ok(Self {
            browser,
            tool_registry: ToolRegistry::with_defaults(),
        })
    }

    /// Connect to an existing browser instance via WebSocket
    pub fn connect(options: ConnectionOptions) -> Result<Self> {
        let browser = Browser::connect(options.ws_url)
            .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            browser,
            tool_registry: ToolRegistry::with_defaults(),
        })
    }

    /// Launch a browser with default options
    pub fn new() -> Result<Self> {
        Self::launch(LaunchOptions::default())
    }

    /// Get the active tab
    pub fn tab(&self) -> Result<Arc<Tab>> {
        self.get_active_tab()
    }

    /// Create a new tab and set it as active
    pub fn new_tab(&mut self) -> Result<Arc<Tab>> {
        let tab = self.browser.new_tab().map_err(|e| {
            BrowserError::TabOperationFailed(format!("Failed to create tab: {}", e))
        })?;
        Ok(tab)
    }

    /// Get all tabs
    pub fn get_tabs(&self) -> Result<Vec<Arc<Tab>>> {
        let tabs = self
            .browser
            .get_tabs()
            .lock()
            .map_err(|e| BrowserError::TabOperationFailed(format!("Failed to get tabs: {}", e)))?
            .clone();

        Ok(tabs)
    }

    /// Get the currently active tab by checking the document visibility and focus state
    pub fn get_active_tab(&self) -> Result<Arc<Tab>> {
        let tabs = self.get_tabs()?;

        // First pass: check for both visibility and focus (strongest signal)
        for tab in &tabs {
            let result = tab.evaluate(
                "document.visibilityState === 'visible' && document.hasFocus()",
                false,
            );
            match result {
                Ok(remote_object) => {
                    if let Some(value) = remote_object.value {
                        if value.as_bool().unwrap_or(false) {
                            return Ok(tab.clone());
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Failed to check tab status: {}", e);
                    continue;
                }
            }
        }

        // Second pass: check just for visibility (weaker signal, but better than nothing)
        for tab in &tabs {
            let result = tab.evaluate("document.visibilityState === 'visible'", false);
            match result {
                Ok(remote_object) => {
                    if let Some(value) = remote_object.value {
                        if value.as_bool().unwrap_or(false) {
                            return Ok(tab.clone());
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Err(BrowserError::TabOperationFailed(
            "No active tab found".to_string(),
        ))
    }

    /// Close the active tab
    pub fn close_active_tab(&mut self) -> Result<()> {
        self.tab()?
            .close(true)
            .map_err(|e| BrowserError::TabOperationFailed(format!("Failed to close tab: {}", e)))?;

        Ok(())
    }

    /// Get the underlying Browser instance
    pub fn browser(&self) -> &Browser {
        &self.browser
    }

    /// Navigate to a URL using the active tab
    pub fn navigate(&self, url: &str) -> Result<()> {
        self.tab()?.navigate_to(url).map_err(|e| {
            BrowserError::NavigationFailed(format!("Failed to navigate to {}: {}", url, e))
        })?;

        Ok(())
    }

    /// Wait for navigation to complete
    pub fn wait_for_navigation(&self) -> Result<()> {
        self.tab()?
            .wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(format!("Navigation timeout: {}", e)))?;

        Ok(())
    }

    /// Extract the DOM tree from the active tab
    pub fn extract_dom(&self) -> Result<DomTree> {
        DomTree::from_tab(&self.tab()?)
    }

    /// Extract the DOM tree with a custom ref prefix (for iframe handling)
    pub fn extract_dom_with_prefix(&self, prefix: &str) -> Result<DomTree> {
        DomTree::from_tab_with_prefix(&self.tab()?, prefix)
    }

    /// Find an element by CSS selector using the provided tab
    pub fn find_element<'a>(
        &self,
        tab: &'a Arc<Tab>,
        css_selector: &str,
    ) -> Result<headless_chrome::Element<'a>> {
        tab.find_element(css_selector).map_err(|e| {
            BrowserError::ElementNotFound(format!("Element '{}' not found: {}", css_selector, e))
        })
    }

    /// Get the tool registry
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }

    /// Get mutable tool registry
    pub fn tool_registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tool_registry
    }

    /// Execute a tool by name
    pub fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<crate::tools::ToolResult> {
        let mut context = ToolContext::new(self);
        self.tool_registry.execute(name, params, &mut context)
    }

    /// Navigate back in browser history
    pub fn go_back(&self) -> Result<()> {
        let go_back_js = r#"
            (function() {
                window.history.back();
                return true;
            })()
        "#;

        self.tab()?
            .evaluate(go_back_js, false)
            .map_err(|e| BrowserError::NavigationFailed(format!("Failed to go back: {}", e)))?;

        // Wait a moment for navigation
        std::thread::sleep(std::time::Duration::from_millis(300));

        Ok(())
    }

    /// Navigate forward in browser history
    pub fn go_forward(&self) -> Result<()> {
        let go_forward_js = r#"
            (function() {
                window.history.forward();
                return true;
            })()
        "#;

        self.tab()?
            .evaluate(go_forward_js, false)
            .map_err(|e| BrowserError::NavigationFailed(format!("Failed to go forward: {}", e)))?;

        // Wait a moment for navigation
        std::thread::sleep(std::time::Duration::from_millis(300));

        Ok(())
    }

    /// Close the browser
    pub fn close(&self) -> Result<()> {
        // Note: The Browser struct doesn't have a public close method in headless_chrome
        // The browser will be closed when the Browser instance is dropped
        // We can close all tabs to effectively shut down
        let tabs = self.get_tabs()?;
        for tab in tabs {
            let _ = tab.close(false); // Ignore errors on individual tab closes
        }
        Ok(())
    }
}

impl Default for BrowserSession {
    fn default() -> Self {
        Self::new().expect("Failed to create default browser session")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_options_builder() {
        let opts = LaunchOptions::new().headless(true).window_size(800, 600);

        assert!(opts.headless);
        assert_eq!(opts.window_width, 800);
        assert_eq!(opts.window_height, 600);
    }

    #[test]
    fn test_connection_options() {
        let opts = ConnectionOptions::new("ws://localhost:9222").timeout(5000);

        assert_eq!(opts.ws_url, "ws://localhost:9222");
        assert_eq!(opts.timeout, 5000);
    }

    #[test]
    #[ignore]
    fn test_get_active_tab() {
        let session = BrowserSession::launch(LaunchOptions::new().headless(true))
            .expect("Failed to launch browser");

        let tab = session.get_active_tab();
        assert!(tab.is_ok());
    }

    // Integration tests (require Chrome to be installed)
    #[test]
    #[ignore] // Ignore by default, run with: cargo test -- --ignored
    fn test_launch_browser() {
        let result = BrowserSession::launch(LaunchOptions::new().headless(true));
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_navigate() {
        let session = BrowserSession::launch(LaunchOptions::new().headless(true))
            .expect("Failed to launch browser");

        let result = session.navigate("about:blank");
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_new_tab() {
        let mut session = BrowserSession::launch(LaunchOptions::new().headless(true))
            .expect("Failed to launch browser");

        let result = session.new_tab();
        assert!(result.is_ok());

        let tabs = session.get_tabs().expect("Failed to get tabs");
        assert!(tabs.len() >= 2);
    }
}
