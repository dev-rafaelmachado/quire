//! Tolerant JSONL for the read-only `gates.jsonl` and `lessons.jsonl` (LOG-01..04).
//!
//! Every non-blank line that is not a JSON object of the expected shape is skipped and counted,
//! including a truncated last line written by an agent mid-append. Unknown fields are ignored,
//! and known fields accept close types (a number as a string, a single `tail` string, …).

use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Jsonl<T> {
    pub items: Vec<T>,
    /// Non-blank lines that could not be read (LOG-04).
    pub skipped: usize,
    /// 1-based line numbers of the skipped lines.
    pub skipped_lines: Vec<usize>,
}

/// Parses one object per line.
pub fn jsonl<T: DeserializeOwned>(text: &str) -> Jsonl<T> {
    let mut out = Jsonl {
        items: Vec::new(),
        skipped: 0,
        skipped_lines: Vec::new(),
    };
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parsed = serde_json::from_str::<Value>(line)
            .ok()
            .filter(Value::is_object)
            .and_then(|v| serde_json::from_value::<T>(v).ok());
        match parsed {
            Some(item) => out.items.push(item),
            None => {
                out.skipped += 1;
                out.skipped_lines.push(i + 1);
            }
        }
    }
    out
}

/// One line of `gates.jsonl`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GateRun {
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub ts: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub repo: Option<String>,
    /// `fase`, `final` or `e2e`.
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub level: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub cmd: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub cwd: Option<String>,
    /// Commit tested; links the run to tasks through the short hash.
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub head: Option<String>,
    #[serde(deserialize_with = "lenient_i64")]
    #[cfg_attr(feature = "specta", specta(type = Option<i32>))]
    pub exit: Option<i64>,
    /// `pass` or `fail`.
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub status: Option<String>,
    #[serde(deserialize_with = "lenient_f64")]
    #[cfg_attr(feature = "specta", specta(type = Option<f64>))]
    pub duration_s: Option<f64>,
    /// Last output lines, usually only on failures (LOG-02).
    #[serde(deserialize_with = "lenient_lines")]
    #[cfg_attr(feature = "specta", specta(type = Vec<String>))]
    pub tail: Vec<String>,
}

impl GateRun {
    /// Failed when the status says so, or when there is no status and the exit code is not 0.
    pub fn failed(&self) -> bool {
        match self.status.as_deref() {
            Some(s) => s.eq_ignore_ascii_case("fail"),
            None => self.exit.is_some_and(|e| e != 0),
        }
    }
}

/// Counts for the gates view header (LOG-01).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GateCounts {
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub runs: usize,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub passes: usize,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub fails: usize,
}

pub fn gate_counts(runs: &[GateRun]) -> GateCounts {
    let fails = runs.iter().filter(|r| r.failed()).count();
    GateCounts {
        runs: runs.len(),
        passes: runs.len() - fails,
        fails,
    }
}

/// Most recent first (LOG-01); runs without timestamp go last, file order kept on ties.
pub fn newest_first(runs: &mut [GateRun]) {
    runs.sort_by(|a, b| match (&a.ts, &b.ts) {
        (Some(x), Some(y)) => y.cmp(x),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
}

/// One line of `lessons.jsonl` (LOG-03).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Lesson {
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub ts: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub feature: Option<String>,
    /// `ac_gap`, `gate_fail`, `spec_precision_gap`, …
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub signal: Option<String>,
    /// `file:line`, a code path or a requirement id.
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub source: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub text: Option<String>,
    #[serde(deserialize_with = "lenient_string")]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub scope: Option<String>,
}

fn lenient_string<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn lenient_i64<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

fn lenient_f64<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

fn lenient_lines<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Array(items) => items
            .into_iter()
            .map(|v| match v {
                Value::String(s) => s,
                other => other.to_string(),
            })
            .collect(),
        Value::String(s) => s.lines().map(str::to_owned).collect(),
        _ => Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_and_truncated_lines_are_skipped_and_counted() {
        let text = "{\"status\":\"pass\",\"exit\":0}\nnot json\n\n[1,2]\n{\"status\":\"fail\",\"exit\":1,\"tail\":[\"x\"]}\n{\"status\":\"pa";
        let r: Jsonl<GateRun> = jsonl(text);
        assert_eq!(r.items.len(), 2);
        assert_eq!(r.skipped, 3);
        assert_eq!(r.skipped_lines, [2, 4, 6]);
        assert_eq!(
            gate_counts(&r.items),
            GateCounts {
                runs: 2,
                passes: 1,
                fails: 1
            }
        );
    }

    #[test]
    fn unknown_fields_and_close_types_are_accepted() {
        let r: Jsonl<GateRun> = jsonl(
            r#"{"ts":"t","exit":"2","duration_s":"1.5","tail":"a\nb","level":7,"extra":{"x":1}}"#,
        );
        assert_eq!(r.skipped, 0);
        let g = &r.items[0];
        assert_eq!(
            (g.exit, g.duration_s, g.level.as_deref()),
            (Some(2), Some(1.5), Some("7"))
        );
        assert_eq!(g.tail, ["a", "b"]);
        assert!(g.failed(), "no status, non-zero exit");
        let l: Jsonl<Lesson> = jsonl(r#"{"signal":"ac_gap","text":"t","source":null,"new":true}"#);
        assert_eq!(l.items[0].signal.as_deref(), Some("ac_gap"));
        assert_eq!(l.items[0].source, None);
    }

    #[test]
    fn sorting_newest_first() {
        let mut runs: Vec<GateRun> = ["2026-01-02T00:00:00Z", "", "2026-01-03T00:00:00Z"]
            .iter()
            .map(|t| GateRun {
                ts: (!t.is_empty()).then(|| (*t).to_owned()),
                ..Default::default()
            })
            .collect();
        newest_first(&mut runs);
        let ts: Vec<Option<&str>> = runs.iter().map(|r| r.ts.as_deref()).collect();
        assert_eq!(
            ts,
            [
                Some("2026-01-03T00:00:00Z"),
                Some("2026-01-02T00:00:00Z"),
                None
            ]
        );
    }

    #[test]
    fn empty_input() {
        let r: Jsonl<Lesson> = jsonl("\n\n");
        assert!(r.items.is_empty());
        assert_eq!(r.skipped, 0);
    }
}
