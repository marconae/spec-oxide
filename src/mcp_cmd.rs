//! MCP command handler for starting the MCP server.

use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};

/// Run the MCP serve command to start an MCP server over stdio.
///
/// This starts an MCP (Model Context Protocol) server that exposes spec tools
/// over stdin/stdout for integration with AI coding assistants.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - The MCP server fails to start
pub fn serve() -> Result<()> {
    // Load configuration
    let config = Config::load(Path::new(".spox/config.toml"))?;

    // Create and run the async runtime for the MCP server
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| Error::Other(format!("Failed to create async runtime: {}", e)))?;

    rt.block_on(async { crate::mcp::run_server(&config).await })
        .map_err(|e| Error::Other(format!("MCP server error: {}", e)))
}

#[cfg(test)]
mod tests {
    // MCP server tests would require mocking stdin/stdout which is complex
    // The actual MCP server functionality is tested in the mcp module
}
