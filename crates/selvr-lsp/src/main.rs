//! `selvr-lsp` — Language Server Protocol server.
//!
//! Communicates over stdin / stdout using the JSON-RPC framing defined by LSP.
//! Start it via:
//!
//! ```text
//! selvr lsp          # alias provided by selvr-cli
//! selvr-lsp          # direct binary
//! ```
//!
//! The VS Code extension invokes this binary as a child process and communicates
//! through its stdin/stdout.

mod server;
mod diagnostics;
mod completion;
mod hover;

use lsp_server::Connection;

fn main() -> anyhow::Result<()> {
    eprintln!("selvr-lsp: starting up");

    // Create an LSP connection over stdin/stdout.
    let (connection, io_threads) = Connection::stdio();

    // Negotiate capabilities with the client.
    let server_capabilities = serde_json::to_value(server::capabilities())?;
    let init_params: lsp_types::InitializeParams =
        serde_json::from_value(connection.initialize(server_capabilities)?)?;

    eprintln!(
        "selvr-lsp: connected to client `{}`",
        init_params.client_info.as_ref().map(|c| c.name.as_str()).unwrap_or("unknown")
    );

    // Enter the main message loop.
    server::run(connection, init_params)?;

    io_threads.join()?;
    eprintln!("selvr-lsp: shutting down");
    Ok(())
}
