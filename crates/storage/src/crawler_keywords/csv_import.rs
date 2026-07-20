//! Parse keyword CSV (kol-nest-server compatible subset).

const MAX_TEXT_LEN: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRow {
    pub text: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub rows: Vec<ParsedRow>,
    pub skipped_too_long: i64,
    pub total_unique: i64,
}

pub fn parse_csv(content: &str) -> ParseResult {
    let raw = content
        .trim()
        .strip_prefix('\u{feff}')
        .unwrap_or(content.trim());
    let lines: Vec<String> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        return ParseResult {
            rows: Vec::new(),
            skipped_too_long: 0,
            total_unique: 0,
        };
    }

    let (delimiter, header, start_index) = detect_header(&lines[0]);
    let idx_text = header.iter().position(|h| h == "text").unwrap_or(0);
    let idx_enabled = header.iter().position(|h| h == "enabled");

    let mut seen = std::collections::HashSet::new();
    let mut rows = Vec::new();
    let mut skipped_too_long = 0i64;

    for line in lines.iter().skip(start_index) {
        let cols: Vec<String> = if let Some(delim) = delimiter {
            line.split(delim).map(|c| c.trim().to_string()).collect()
        } else {
            vec![line.trim().to_string()]
        };
        let text = cols
            .get(idx_text)
            .map(|s| s.trim())
            .unwrap_or("")
            .to_string();
        if text.is_empty() {
            continue;
        }
        if text.len() > MAX_TEXT_LEN {
            skipped_too_long += 1;
            continue;
        }
        let enabled = if let Some(idx) = idx_enabled {
            let en = cols
                .get(idx)
                .map(|s| s.trim().to_lowercase())
                .unwrap_or_default();
            en == "1" || en == "true" || en == "yes"
        } else {
            true
        };
        if seen.insert(text.clone()) {
            rows.push(ParsedRow { text, enabled });
        }
    }

    let total_unique = rows.len() as i64;
    ParseResult {
        rows,
        skipped_too_long,
        total_unique,
    }
}

fn detect_header(first_line: &str) -> (Option<char>, Vec<String>, usize) {
    for delim in [',', '\t', ';'] {
        let parts: Vec<String> = first_line
            .split(delim)
            .map(|h| h.trim().to_lowercase())
            .collect();
        if parts.len() > 1 && parts.iter().any(|h| h == "text") {
            return (Some(delim), parts, 1);
        }
    }
    let single = first_line.trim().to_lowercase();
    if single == "text" {
        return (None, vec!["text".to_string()], 1);
    }
    (None, vec!["text".to_string()], 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_header_csv() {
        let result = parse_csv("text,enabled\nfoo,true\nbar,0\nfoo,1");
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].text, "foo");
        assert!(result.rows[0].enabled);
        assert!(!result.rows[1].enabled);
    }

    #[test]
    fn parses_single_column() {
        let result = parse_csv("alpha\nbeta\nalpha");
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].text, "alpha");
    }
}
