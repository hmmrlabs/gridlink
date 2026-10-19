# Hammer Sheets MCP (GridLink)

**Turn an AI conversation into a spreadsheet you can share.**

Hammer Sheets MCP is powered by GridLink, a small Rust MCP server that creates links to editable GridCraft spreadsheets. It runs locally over stdio or inside a hosted Streamable HTTP transport. No database, accounts, file uploads, or document storage are required.

## Install

Rust 1.95 or newer is required.

```sh
cargo install --git https://github.com/hmmrlabs/gridlink --locked --bin gridlink
```

Add to a local MCP client, such as Claude Desktop:

```json
{
  "mcpServers": {
    "gridlink": {
      "command": "/absolute/path/to/gridlink"
    }
  }
}
```

Use the path printed by `command -v gridlink` (`where gridlink` on Windows). Run the binary without arguments; it speaks newline-delimited MCP JSON-RPC on stdin/stdout. Diagnostics go to stderr.

Set `GRIDLINK_VIEWER_URL` to use another HTTPS viewer origin. The default is `https://sheets.hammerlabs.ai`.

For remote MCP clients, connect to `https://sheets.hammerlabs.ai/mcp` once that deployment is live. Remote connector availability depends on the client and plan. Unlike local mode, a hosted transport sees tool inputs while processing them; the provided service does not store them.

## Tools

| Tool | Input | Result |
| --- | --- | --- |
| `create_spreadsheet` | Workbook name and sheets containing names and rows | Self-contained `#sheet=` link with numbers, text, booleans and formulas |
| `create_viewer_link` | Existing absolute HTTPS workbook URL | `#url=` viewer link; no server fetch |
| `share_workbook` | Standard base64 of a small XLSX package | Self-contained `#xlsx=` link |

Example `create_spreadsheet` arguments:

```json
{
  "name": "Launch budget",
  "sheets": [{
    "name": "Budget",
    "rows": [
      ["Category", "Amount"],
      ["Design", 1200],
      ["Development", 3400],
      ["Launch", 800],
      ["Total", "=SUM(B2:B4)"]
    ]
  }]
}
```

Ask your assistant: “Use GridLink to create a launch budget with formulas and give me the editable link.”

The browser builds the workbook and computes formulas using GridCraft WASM. After edits, save a local draft, download XLSX, or generate a new share link. Shared links are snapshots, not collaborative live documents.

## Privacy and limits

- Local MCP performs no network requests. Your AI provider still sees the data in your conversation.
- Fragment payloads are not part of HTTP requests. Anyone with the complete link can read the sheet; links are not encrypted, cannot be revoked, and may remain in chat or browser history.
- Inline payloads are limited to **48 KiB** before base64. Some clients truncate long URLs. For larger files, use an existing HTTPS source and `create_viewer_link`.
- Remote sources must permit CORS. Cookies and embedded credentials are not supported.
- Generated sheets: at most 16 sheets, 1,000 rows per sheet, 100 columns, 10,000 cells total. Formula strings start with `=`.
- Protocol messages: 128 KiB maximum. Oversized, malformed and unsupported requests return errors without crashing.
- The library implements MCP versions `2025-11-25`, `2025-06-18` and `2025-03-26`. It supports initialization, ping, tool listing and tool calls. No resources, prompts, subscriptions or server-initiated requests.
- `share_workbook` validates size and ZIP signature; workbook parsing happens in the viewer. Invalid packages show an import error there.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`src/mcp.rs` is transport-independent. Hosted applications can call `gridlink::mcp::handle`; they must supply HTTP origin validation, bounded request reading, protocol headers and their own privacy policy.

Licensed under MIT OR Apache-2.0. No vendor artwork or spreadsheet files are bundled.
