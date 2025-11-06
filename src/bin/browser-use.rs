//! Browser-use MCP Server
//!
//! This binary provides a Model Context Protocol (MCP) server for browser automation.
//! It exposes browser automation tools that can be used by AI assistants and other MCP clients.

use browser_use::browser::LaunchOptions;
use browser_use::mcp::BrowserServer;
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use std::io::{stdin, stdout};

#[derive(Parser)]
#[command(name = "browser-use")]
#[command(version)]
#[command(about = "Browser automation MCP server", long_about = None)]
struct Cli {
    /// Launch browser in headed mode (default: headless)
    #[arg(long, short = 'H')]
    headed: bool,

    /// Path to custom browser executable
    #[arg(long, value_name = "PATH")]
    executable_path: Option<String>,

    /// CDP endpoint URL for remote browser connection
    #[arg(long, value_name = "URL")]
    cdp_endpoint: Option<String>,

    /// WebSocket endpoint URL for remote browser connection
    #[arg(long, value_name = "URL")]
    ws_endpoint: Option<String>,

    /// Persistent browser profile directory
    #[arg(long, value_name = "DIR")]
    user_data_dir: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Configure browser launch options
    let options = LaunchOptions {
        headless: !cli.headed,
        ..Default::default()
    };

    // Create browser server
    let service = BrowserServer::with_options(options.clone())
        .map_err(|e| format!("Failed to create browser server: {}", e))?;

    eprintln!("Browser-use MCP Server v{}", env!("CARGO_PKG_VERSION"));
    eprintln!(
        "Browser mode: {}",
        if options.headless {
            "headless"
        } else {
            "headed"
        }
    );

    if let Some(ref path) = cli.executable_path {
        eprintln!("Browser executable: {}", path);
    }

    if let Some(ref endpoint) = cli.cdp_endpoint {
        eprintln!("CDP endpoint: {}", endpoint);
    }

    if let Some(ref endpoint) = cli.ws_endpoint {
        eprintln!("WebSocket endpoint: {}", endpoint);
    }

    if let Some(ref dir) = cli.user_data_dir {
        eprintln!("User data directory: {}", dir);
    }

    eprintln!("Ready to accept MCP connections via stdio");

    // Start stdio transport
    let (_read, _write) = (stdin(), stdout());
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
