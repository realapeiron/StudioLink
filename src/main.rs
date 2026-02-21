mod error;
mod mcp;
mod server;
mod state;
mod tools;

use clap::Parser;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

/// StudioLink — Advanced Roblox Studio MCP Server
/// 36 tools for professional game development with AI assistance
#[derive(Parser, Debug)]
#[command(name = "studiolink", version, about)]
struct Args {
    /// HTTP server port for Studio plugin communication
    #[arg(short, long, default_value_t = 34872)]
    port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    // Initialize logging (stderr only — stdout is for MCP JSON-RPC)
    let filter = if args.verbose {
        EnvFilter::new("studiolink=debug,tower_http=debug")
    } else {
        EnvFilter::new("studiolink=info")
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!(
        "StudioLink v{} — Advanced Roblox Studio MCP Server",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("36 tools for professional game development");

    // Create shared state
    let (state, notify_rx) = state::AppState::new();

    // Try to start HTTP server — if port is taken, switch to proxy mode
    let port = args.port;
    let proxy_url = format!("http://127.0.0.1:{}", port);

    // Check if port is available by trying to bind
    match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
        Ok(listener) => {
            // Port available — we are the primary instance
            tracing::info!("Primary mode: starting HTTP server on port {}", port);
            let http_state = state.clone();
            tokio::spawn(async move {
                let router = server::create_router(http_state, notify_rx);
                if let Err(e) = axum::serve(listener, router).await {
                    tracing::error!("HTTP server error: {}", e);
                }
            });
        }
        Err(_) => {
            // Port taken — another StudioLink instance is running, switch to proxy mode
            tracing::info!("Proxy mode: forwarding tool calls to primary server at {}", proxy_url);
            let mut s = state.lock().await;
            s.proxy_mode = true;
            s.proxy_url = proxy_url;
            drop(s);
        }
    }

    // Start MCP server on stdio
    tracing::info!("Starting MCP server on stdio...");
    let mcp_handler = mcp::StudioLinkMcp::new(state);

    // Run MCP server via stdio transport — this is the main loop
    let transport = rmcp::transport::stdio();
    let mcp_server = mcp_handler.serve(transport).await?;

    // Wait for MCP server to finish (HTTP server runs independently in background)
    match mcp_server.waiting().await {
        Ok(_) => tracing::info!("MCP server stopped gracefully"),
        Err(e) => tracing::error!("MCP server error: {}", e),
    }

    Ok(())
}
