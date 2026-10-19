//! Stateless spreadsheet sharing tools for local and hosted MCP.
pub mod mcp;
pub mod share;

/// Credentials and arbitrary browser URL schemes are never accepted.
pub fn source_url(input: &str) -> Result<url::Url, String> {
    if input.len() > 8192 {
        return Err("Source URL is too long.".into());
    }
    let url = url::Url::parse(input).map_err(|_| "Enter an absolute HTTPS workbook URL.")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Use an HTTPS URL without embedded credentials.".into());
    }
    Ok(url)
}
