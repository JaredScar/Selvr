//! `selvr-dap` — Debug Adapter Protocol server for Selvr.
//!
//! Runs over stdin/stdout using the DAP JSON framing:
//!
//!   Content-Length: <n>\r\n
//!   \r\n
//!   <json payload>
//!
//! The VS Code extension starts this binary as a child process when the user
//! presses F5 on a `.self` file.
//!
//! **Targeting-aware debugging**:
//! - JS-targeted functions appear in normal stack frames.
//! - WASM-targeted functions are annotated with `presentationHint: "wasm"`.
//! - The IDE can colour/filter these distinctly.

mod protocol;
mod adapter;

use std::io::{self, BufRead, Read, Write};
use protocol::DapMessage;
use adapter::Adapter;

fn main() -> anyhow::Result<()> {
    eprintln!("selvr-dap: starting");

    let stdin  = io::stdin();
    let stdout = io::stdout();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    let mut state  = Adapter::new();

    loop {
        // Read Content-Length header.
        let mut header_line = String::new();
        let n = reader.read_line(&mut header_line)?;
        if n == 0 { break; }  // EOF

        let header_line = header_line.trim();
        if header_line.is_empty() { continue; }

        let content_length: usize = if let Some(rest) = header_line.strip_prefix("Content-Length: ") {
            rest.trim().parse().unwrap_or(0)
        } else {
            continue; // skip unknown headers
        };

        // Skip the blank line between header and body.
        let mut blank = String::new();
        reader.read_line(&mut blank)?;

        // Read the JSON body.
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;

        let msg: DapMessage = match serde_json::from_slice(&body) {
            Ok(m)  => m,
            Err(e) => { eprintln!("selvr-dap: parse error: {e}"); continue; }
        };

        let responses = match msg {
            DapMessage::Request(req) => state.handle(&req),
            _ => vec![],
        };

        for resp in responses {
            let json = serde_json::to_string(&resp)?;
            write!(writer, "Content-Length: {}\r\n\r\n{}", json.len(), json)?;
            writer.flush()?;
        }
    }

    eprintln!("selvr-dap: exiting");
    Ok(())
}
