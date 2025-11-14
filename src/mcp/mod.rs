//! MCP (Model Context Protocol) server implementation for browser automation
//!
//! This module provides rmcp-compatible tools by wrapping the existing tool implementations.

pub mod handler;
pub use handler::BrowserServer;

use crate::tools::{self, Tool, ToolContext, ToolResult as InternalToolResult};
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router,
};

/// Convert internal ToolResult to MCP CallToolResult
fn convert_result(result: InternalToolResult) -> Result<CallToolResult, McpError> {
    if result.success {
        let text = if let Some(data) = result.data {
            serde_json::to_string_pretty(&data).unwrap_or_else(|_| data.to_string())
        } else {
            "Success".to_string()
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    } else {
        let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
        Err(McpError::internal_error(error_msg, None))
    }
}

/// Macro to register MCP tools by automatically generating wrapper functions
macro_rules! register_mcp_tools {
    ($($mcp_name:ident => $tool_type:ty, $description:expr);* $(;)?) => {
        #[tool_router]
        impl BrowserServer {
            $(
                #[tool(description = $description)]
                fn $mcp_name(
                    &self,
                    params: Parameters<<$tool_type as Tool>::Params>,
                ) -> Result<CallToolResult, McpError> {
                    let session = self.session();
                    let mut context = ToolContext::new(&*session);
                    let tool = <$tool_type>::default();
                    let result = tool.execute_typed(params.0, &mut context)
                        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                    convert_result(result)
                }
            )*
        }
    };
}

// 以下未实现：
// browser_go_back
// browser_go_forward
// browser_new_tab
// browser_tab_list
// browser_switch_tab
// browser_close_tab
// browser_get_download_list
// browser_select
// browser_hover
// browser_scroll
// browser_close

// Register all MCP tools using the macro
register_mcp_tools! {
    browser_navigate => tools::navigate::NavigateTool, "Navigate to a specified URL in the browser";
    browser_click => tools::click::ClickTool, "Click on an element specified by CSS selector or index";
    browser_form_input_fill => tools::input::InputTool, "Type text into an input element";
    browser_get_text => tools::extract::ExtractContentTool, "Extract text or HTML content from the page or an element";
    browser_screenshot => tools::screenshot::ScreenshotTool, "Capture a screenshot of the current page";
    browser_evaluate => tools::evaluate::EvaluateTool, "Execute JavaScript code in the browser context";
    browser_wait => tools::wait::WaitTool, "Wait for an element to appear on the page";
    browser_get_markdown => tools::markdown::GetMarkdownTool, "Get the markdown content of the current page";
    browser_read_links => tools::read_links::ReadLinksTool, "Read all links on the current page";
    browser_get_clickable_elements => tools::get_clickable_elements::GetClickableElementsTool, "Get all clickable/interactive elements on the page";
    browser_press_key => tools::press_key::PressKeyTool, "Press a key on the keyboard";
}
