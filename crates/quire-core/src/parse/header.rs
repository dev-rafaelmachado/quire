//! `**Chave:** valor` metadata at the top of `spec.md` / `summary.md` (VIW-07).

use super::bold_field;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderField {
    pub key: String,
    /// Raw markdown value.
    pub value: String,
    /// Value with markdown links reduced to their text and code spans unwrapped.
    pub text: String,
    /// First link target in the value (markdown link or bare http(s) URL).
    pub url: Option<String>,
    pub line: usize,
}

/// Reads the block of `**Key:** value` lines after the optional `# title`.
///
/// Leading blank lines and the title are skipped; the block ends at the first blank line,
/// heading or non-field line after it starts (docs/spec-format.md).
pub fn header(md: &str) -> Vec<HeaderField> {
    let mut out = Vec::new();
    let mut started = false;
    for (i, line) in md.lines().enumerate() {
        let t = line.trim();
        if !started && (t.is_empty() || (t.starts_with("# ") && out.is_empty()) || t == "#") {
            continue;
        }
        if t.starts_with('#') {
            break;
        }
        match bold_field(t) {
            Some((key, value)) if !key.is_empty() => {
                started = true;
                out.push(HeaderField {
                    key: key.to_owned(),
                    value: value.to_owned(),
                    text: plain_text(value),
                    url: first_url(value),
                    line: i + 1,
                });
            }
            _ => break,
        }
    }
    out
}

/// `[text](url)` -> `text`, `` `code` `` -> `code`.
fn plain_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('[') {
        let Some(mid) = rest[start..].find("](").map(|m| start + m) else {
            break;
        };
        let Some(end) = rest[mid..].find(')').map(|e| mid + e) else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push_str(&rest[start + 1..mid]);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out.replace('`', "")
}

fn first_url(value: &str) -> Option<String> {
    if let Some(mid) = value.find("](") {
        let after = &value[mid + 2..];
        if let Some(end) = after.find(')') {
            let url = after[..end].trim();
            if !url.is_empty() {
                return Some(url.to_owned());
            }
        }
    }
    let start = value.find("https://").or_else(|| value.find("http://"))?;
    let url: String = value[start..]
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != ')' && *c != '>')
        .collect();
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "# ACME-142 — Tokens\n\n**Status:** Concluído · 7/8 tarefas\n**Issue:** [ACME-142](https://tracker.example/browse/ACME-142)\n**Idioma:** pt-BR\n\n## Fontes\n\n**Not:** header\n";

    #[test]
    fn reads_the_block_after_the_title() {
        let h = header(SPEC);
        let keys: Vec<&str> = h.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, ["Status", "Issue", "Idioma"]);
        assert_eq!(h[1].text, "ACME-142");
        assert_eq!(
            h[1].url.as_deref(),
            Some("https://tracker.example/browse/ACME-142")
        );
        assert_eq!(h[1].line, 4);
        assert_eq!(h[0].url, None);
    }

    #[test]
    fn tolerates_missing_title_colon_outside_and_bare_urls() {
        let h = header("**Spec**: `spec.md`\n**Link:** see https://x.example/a?b=1 now\n\ntext");
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].text, "spec.md");
        assert_eq!(h[1].url.as_deref(), Some("https://x.example/a?b=1"));
    }

    #[test]
    fn out_of_format_is_empty() {
        assert!(header("").is_empty());
        assert!(header("# Title\n\nJust prose.\n**Late:** field").is_empty());
        assert!(header("## Section first\n**K:** v").is_empty());
        assert!(header("---\ntitle: x\n---\n").is_empty());
    }
}
