//! Local MCP over newline-delimited JSON-RPC. stdout is protocol-only.
use gridlink::{mcp, share::MAX_MESSAGE_BYTES};
use std::io::{self, BufRead, Write};

fn main() {
    if let Err(error) = serve() {
        eprintln!("GridLink MCP: {error}");
        std::process::exit(1);
    }
}
fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("GRIDLINK_VIEWER_URL")
        .unwrap_or_else(|_| "https://sheets.hammerlabs.ai".into());
    gridlink::source_url(&base)?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        // Bound allocation even if a hostile client never writes a newline.
        let mut line = Vec::new();
        let mut oversized = false;
        loop {
            let chunk = input.fill_buf()?;
            if chunk.is_empty() {
                break;
            }
            let count = chunk
                .iter()
                .position(|b| *b == b'\n')
                .map_or(chunk.len(), |i| i + 1);
            let ends = chunk.get(count.saturating_sub(1)) == Some(&b'\n');
            if line.len().saturating_add(count) > MAX_MESSAGE_BYTES {
                oversized = true;
            }
            if !oversized {
                line.extend_from_slice(chunk.get(..count).ok_or("Invalid input buffer")?);
            }
            input.consume(count);
            if ends {
                break;
            }
        }
        if line.is_empty() && !oversized {
            break;
        }
        let response = if oversized {
            Some(mcp::error(
                serde_json::Value::Null,
                -32600,
                "Message exceeds 128 KiB",
            ))
        } else {
            match serde_json::from_slice(&line) {
                Ok(value) => mcp::handle(value, &base),
                Err(_) => Some(mcp::error(serde_json::Value::Null, -32700, "Parse error")),
            }
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut output, &response)?;
            output.write_all(b"\n")?;
            output.flush()?;
        }
    }
    Ok(())
}
