//! Browser-use MCP Server
//!
//! This binary provides a Model Context Protocol (MCP) server for browser automation.
//! It exposes browser automation tools that can be used by AI assistants and other MCP clients.

use browser_use::browser::LaunchOptions;
use browser_use::mcp::BrowserServer;
use clap::{Parser, ValueEnum};
use rmcp::{ServiceExt, transport::stdio};
use std::io::{stdin, stdout};

#[cfg(feature = "mcp-server")]
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Transport {
    /// Standard input/output transport (default)
    Stdio,
    /// HTTP streamable transport
    Http,
}

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

    /// Transport type to use
    #[arg(long, short = 't', value_enum, default_value = "stdio")]
    transport: Transport,

    /// Port for HTTP transport (default: 3000)
    #[arg(long, short = 'p', default_value = "3000")]
    port: u16,

    /// HTTP streamable endpoint path (default: /mcp)
    #[arg(long, default_value = "/mcp")]
    http_path: String,
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

    // Route to appropriate transport
    match cli.transport {
        Transport::Stdio => {
            eprintln!("Transport: stdio");
            eprintln!("Ready to accept MCP connections via stdio");
            let (_read, _write) = (stdin(), stdout());
            let server = service.serve(stdio()).await?;
            server.waiting().await?;
        }
        Transport::Http => {
            eprintln!("Transport: HTTP streamable");
            eprintln!("Port: {}", cli.port);
            eprintln!("HTTP path: {}", cli.http_path);

            let bind_addr = format!("127.0.0.1:{}", cli.port);

            // Create service factory closure
            let service_factory = move || {
                BrowserServer::with_options(options.clone())
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            };

            let http_service = StreamableHttpService::new(
                service_factory,
                LocalSessionManager::default().into(),
                Default::default(),
            );

            let router = axum::Router::new().nest_service(&cli.http_path, http_service);

            eprintln!(
                "Ready to accept MCP connections at http://{}{}",
                bind_addr, cli.http_path
            );

            let listener = tokio::net::TcpListener::bind(bind_addr).await?;
            axum::serve(listener, router).await?;
        }
    }

    Ok(())
}
