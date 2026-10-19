//! Self-contained links. Fragment data is never sent in HTTP requests.
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_SHARE_BYTES: usize = 48 * 1024;
pub const MAX_MESSAGE_BYTES: usize = 128 * 1024;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<Value>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spreadsheet {
    pub name: String,
    pub sheets: Vec<Sheet>,
}
impl Spreadsheet {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || self.name.len() > 160
            || self.sheets.is_empty()
            || self.sheets.len() > 16
        {
            return Err("Supply a name (1–160 bytes) and 1–16 sheets.".into());
        }
        let mut names = std::collections::HashSet::new();
        let mut count = 0usize;
        for sheet in &self.sheets {
            if sheet.name.is_empty()
                || sheet.name.chars().count() > 31
                || sheet.name.contains(['[', ']', ':', '*', '?', '/', '\\'])
                || !names.insert(sheet.name.to_lowercase())
            {
                return Err(
                    "Sheet names must be unique, 1–31 characters, without []:*?/\\.".into(),
                );
            }
            if sheet.rows.len() > 1000 {
                return Err("At most 1,000 rows per sheet.".into());
            }
            for row in &sheet.rows {
                count = count.saturating_add(row.len());
                if row.len() > 100 || count > 10_000 {
                    return Err("At most 100 columns and 10,000 total cells.".into());
                }
                for cell in row {
                    if cell.is_array()
                        || cell.is_object()
                        || cell.as_str().is_some_and(|s| s.len() > 8192)
                    {
                        return Err("Cells must be numbers, booleans, null or strings up to 8,192 bytes. Use = for formulas.".into());
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn encoded(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() > MAX_SHARE_BYTES {
        return Err("Sheet is too large for an inline link (48 KiB maximum). Host the workbook yourself and use create_viewer_link.".into());
    }
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
pub fn decoded(text: &str) -> Result<Vec<u8>, String> {
    if text.len() > MAX_SHARE_BYTES.div_ceil(3) * 4 {
        return Err("Share link is too large.".into());
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(text)
        .map_err(|_| "Share link is damaged or truncated.")?;
    if bytes.len() > MAX_SHARE_BYTES {
        return Err("Share link is too large.".into());
    }
    Ok(bytes)
}
pub fn spreadsheet(text: &str) -> Result<Spreadsheet, String> {
    let spec: Spreadsheet = serde_json::from_slice(&decoded(text)?)
        .map_err(|_| "Spreadsheet link is damaged or unsupported.")?;
    spec.validate()?;
    Ok(spec)
}
pub fn link(base: &str, field: &str, value: &str) -> Result<String, String> {
    let mut base = crate::source_url(base)?;
    base.set_path("/grid");
    base.set_query(None);
    let fragment = url::form_urlencoded::Serializer::new(String::new())
        .append_pair(field, value)
        .finish();
    base.set_fragment(Some(&fragment));
    Ok(base.into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn share_roundtrip_stays_in_fragment() {
        let spec = Spreadsheet {
            name: "Budget".into(),
            sheets: vec![Sheet {
                name: "Plan".into(),
                rows: vec![vec![Value::from(21), Value::from("=A1*2")]],
            }],
        };
        spec.validate().unwrap();
        let encoded = encoded(&serde_json::to_vec(&spec).unwrap()).unwrap();
        let url = url::Url::parse(&link("https://view.hammerlabs.ai", "sheet", &encoded).unwrap())
            .unwrap();
        assert!(url.query().is_none());
        let decoded_spec = spreadsheet(&encoded).unwrap();
        assert_eq!(decoded_spec.sheets[0].rows[0][1], "=A1*2");
        assert_eq!(url.path(), "/grid");
    }
    #[test]
    fn rejects_excess_and_malformed_input() {
        assert!(encoded(&vec![0; MAX_SHARE_BYTES + 1]).is_err());
        assert!(decoded("%").is_err());
        let spec = Spreadsheet {
            name: "Budget".into(),
            sheets: vec![Sheet {
                name: "Bad/name".into(),
                rows: vec![],
            }],
        };
        assert!(spec.validate().is_err());
    }
}
