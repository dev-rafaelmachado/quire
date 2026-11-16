//! Requirement ids declared in HTML comments, e.g. `<!-- TOK-01 -->` (docs/spec-format.md).

use regex::Regex;

use super::Fence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdRef {
    pub id: String,
    pub line: usize,
}

/// Ids matched by `pattern` inside `<!-- … -->` comments, outside code blocks, in order.
pub fn ids(md: &str, pattern: &Regex) -> Vec<IdRef> {
    let mut out = Vec::new();
    let mut fence = Fence::default();
    for (i, line) in md.lines().enumerate() {
        if fence.step(line) {
            continue;
        }
        let mut rest = line;
        while let Some(start) = rest.find("<!--") {
            let Some(end) = rest[start..].find("-->").map(|e| start + e) else {
                break;
            };
            for m in pattern.find_iter(&rest[start + 4..end]) {
                out.push(IdRef {
                    id: m.as_str().to_owned(),
                    line: i + 1,
                });
            }
            rest = &rest[end + 3..];
        }
    }
    out
}

/// All ids matched by `pattern` in free text (e.g. a `**Requirement**` field).
pub fn ids_in_text(text: &str, pattern: &Regex) -> Vec<String> {
    pattern
        .find_iter(text)
        .map(|m| m.as_str().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn re() -> Regex {
        Regex::new(r"[A-Z]{2,5}-\d{2}").unwrap()
    }

    #[test]
    fn finds_ids_in_comments_only() {
        let md = "1. QUANDO x ENTÃO y. <!-- TOK-01 -->\nTOK-02 in prose\n```\n<!-- TOK-03 -->\n```\n<!-- TOK-04 --> and <!-- RTL-01 -->";
        let got: Vec<(String, usize)> =
            ids(md, &re()).into_iter().map(|r| (r.id, r.line)).collect();
        assert_eq!(
            got,
            [
                ("TOK-01".into(), 1),
                ("TOK-04".into(), 6),
                ("RTL-01".into(), 6)
            ]
        );
    }

    #[test]
    fn empty_and_unclosed() {
        assert!(ids("", &re()).is_empty());
        assert!(ids("<!-- TOK-01", &re()).is_empty());
        assert_eq!(
            ids_in_text("contrato (suporta DSC-04, DUP-05)", &re()),
            ["DSC-04", "DUP-05"]
        );
    }
}
