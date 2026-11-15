//! `tasks.md` grammar of the `phases` preset (docs/spec-format.md, TSK-01..04).
//!
//! - Phases: `### Fase <n>: <title>` with an optional ` — <repo>` suffix. The first fenced
//!   block right after the heading is the dependency diagram (`T1 → T2`, one chain per line).
//! - Task lines: `- [ ] T<n>: title` / `- [x] T<n>: title (abc1234)`; an annotation between the
//!   id and the colon (`T4 (waiting CI):`) is kept as `note`.
//! - Details: `### T<n>: title` sections with `**Field**: value` lines; a field with an empty
//!   value collects the list items below it (`**Pronto quando**:`).
//!
//! Checkbox lines before any phase go to an implicit phase without number. Lines inside code
//! blocks are ignored, except the diagram.

use std::sync::OnceLock;

use regex::Regex;

use super::{Fence, bold_field};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub done: bool,
    /// Short commit hash at the end of the line, if any.
    pub hash: Option<String>,
    /// Text between the id and the colon, e.g. `aguardando CI no GitHub`.
    pub note: Option<String>,
    /// Requirement ids from the task's detail section (TSK-03).
    pub requirements: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    Complete,
    InProgress,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase {
    /// `None` for the implicit phase that holds tasks listed before any phase heading.
    pub number: Option<u32>,
    pub title: String,
    pub repo: Option<String>,
    /// Dependency chains, one per diagram line, ids in order (TSK-02).
    pub diagram: Vec<Vec<String>>,
    pub tasks: Vec<Task>,
    pub line: usize,
}

impl Phase {
    pub fn done(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    pub fn total(&self) -> usize {
        self.tasks.len()
    }

    /// TSK-01: complete when every task is done, in progress when some are.
    pub fn state(&self) -> PhaseState {
        match (self.done(), self.total()) {
            (_, 0) | (0, _) => PhaseState::Pending,
            (d, t) if d == t => PhaseState::Complete,
            _ => PhaseState::InProgress,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailField {
    pub name: String,
    pub text: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDetail {
    pub id: String,
    pub title: String,
    pub fields: Vec<DetailField>,
    pub line: usize,
}

impl TaskDetail {
    /// Field by name, case-insensitive (`Onde`, `Depende de`, `Pronto quando`, …).
    pub fn field(&self, name: &str) -> Option<&DetailField> {
        self.fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
    }

    /// Task ids in `**Depende de**` (`nenhuma` gives an empty list).
    pub fn depends_on(&self) -> Vec<String> {
        self.field("Depende de")
            .map(|f| task_ids(&f.text))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TasksDoc {
    pub phases: Vec<Phase>,
    pub details: Vec<TaskDetail>,
}

impl TasksDoc {
    /// (done, total) over every checkbox line.
    pub fn progress(&self) -> (usize, usize) {
        self.phases
            .iter()
            .fold((0, 0), |(d, t), p| (d + p.done(), t + p.total()))
    }

    pub fn detail(&self, id: &str) -> Option<&TaskDetail> {
        self.details.iter().find(|d| d.id == id)
    }

    pub fn task(&self, id: &str) -> Option<&Task> {
        self.phases
            .iter()
            .flat_map(|p| &p.tasks)
            .find(|t| t.id == id)
    }

    pub fn is_empty(&self) -> bool {
        self.phases.is_empty() && self.details.is_empty()
    }
}

struct Patterns {
    phase: Regex,
    task: Regex,
    hash: Regex,
    detail: Regex,
    task_id: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        // Constant patterns, exercised by every test of this module: failing here is a bug in
        // the source, not a runtime condition.
        #[allow(clippy::expect_used)]
        let re = |s: &str| Regex::new(s).expect("static tasks.md pattern");
        Patterns {
            phase: re(r"^###\s+Fase\s+(\d+)\s*:\s*(.+?)\s*$"),
            task: re(r"^\s*[-*]\s+\[([ xX])\]\s+(T\d+)\b(?:\s*\(([^)]*)\))?\s*:\s*(.*?)\s*$"),
            hash: re(r"^(.*?)\s*\(([0-9a-f]{7,40})\)$"),
            detail: re(r"^###\s+(T\d+)\s*:\s*(.+?)\s*$"),
            task_id: re(r"\bT\d+\b"),
        }
    })
}

fn task_ids(text: &str) -> Vec<String> {
    patterns()
        .task_id
        .find_iter(text)
        .map(|m| m.as_str().to_owned())
        .collect()
}

/// Parses `tasks.md`. `id_pattern` extracts requirement ids from `**Requirement**`; without
/// it the field is split on commas.
pub fn tasks(md: &str, id_pattern: Option<&Regex>) -> TasksDoc {
    let p = patterns();
    let mut doc = TasksDoc::default();
    let mut fence = Fence::default();
    // Phase index that may still receive its diagram (heading seen, no task yet).
    let mut awaiting_diagram: Option<usize> = None;
    let mut diagram_open: Option<usize> = None;
    let mut in_phase: Option<usize> = None;
    let mut detail: Option<TaskDetail> = None;

    for (i, raw) in md.lines().enumerate() {
        let line_no = i + 1;
        let was_open = fence.is_open();
        if fence.step(raw) {
            if !was_open {
                diagram_open = awaiting_diagram.take();
            } else if !fence.is_open() {
                diagram_open = None;
            } else if let Some(pi) = diagram_open {
                let chain = task_ids(raw);
                if let (false, Some(phase)) = (chain.is_empty(), doc.phases.get_mut(pi)) {
                    phase.diagram.push(chain);
                }
            }
            continue;
        }
        let t = raw.trim();
        if t.starts_with('#') {
            if let Some(d) = detail.take() {
                doc.details.push(d);
            }
            awaiting_diagram = None;
            in_phase = None;
            if let Some(c) = p.phase.captures(t) {
                let number = c.get(1).and_then(|m| m.as_str().parse().ok());
                let full = c.get(2).map_or("", |m| m.as_str());
                let (title, repo) = match full.split_once(" — ") {
                    Some((a, b)) => (a.trim().to_owned(), Some(b.trim().to_owned())),
                    None => (full.to_owned(), None),
                };
                doc.phases.push(Phase {
                    number,
                    title,
                    repo,
                    diagram: vec![],
                    tasks: vec![],
                    line: line_no,
                });
                in_phase = Some(doc.phases.len() - 1);
                awaiting_diagram = in_phase;
            } else if let Some(c) = p.detail.captures(t) {
                detail = Some(TaskDetail {
                    id: c[1].to_owned(),
                    title: c[2].to_owned(),
                    fields: vec![],
                    line: line_no,
                });
            }
            continue;
        }
        if let Some(d) = detail.as_mut() {
            if t == "---" {
                if let Some(d) = detail.take() {
                    doc.details.push(d);
                }
                continue;
            }
            if let Some((name, value)) = bold_field(t) {
                d.fields.push(DetailField {
                    name: name.to_owned(),
                    text: value.to_owned(),
                    items: vec![],
                });
            } else if let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
                if let Some(f) = d.fields.last_mut() {
                    f.items.push(item.trim().to_owned());
                }
            } else if let (false, Some(f)) = (t.is_empty(), d.fields.last_mut()) {
                if !f.text.is_empty() {
                    f.text.push(' ');
                }
                f.text.push_str(t);
            }
            continue;
        }
        if let Some(c) = p.task.captures(raw) {
            let rest = c.get(4).map_or("", |m| m.as_str());
            let (title, hash) = match p.hash.captures(rest) {
                Some(h) => (h[1].to_owned(), Some(h[2].to_owned())),
                None => (rest.to_owned(), None),
            };
            let task = Task {
                id: c[2].to_owned(),
                title,
                done: &c[1] != " ",
                hash,
                note: c
                    .get(3)
                    .map(|m| m.as_str().trim().to_owned())
                    .filter(|s| !s.is_empty()),
                requirements: vec![],
                line: line_no,
            };
            let pi = match in_phase {
                Some(pi) => pi,
                None => {
                    // Tasks before any phase heading go to an implicit phase at the top.
                    if doc.phases.first().is_none_or(|ph| ph.number.is_some()) {
                        doc.phases.insert(
                            0,
                            Phase {
                                number: None,
                                title: String::new(),
                                repo: None,
                                diagram: vec![],
                                tasks: vec![],
                                line: line_no,
                            },
                        );
                    }
                    0
                }
            };
            if let Some(phase) = doc.phases.get_mut(pi) {
                phase.tasks.push(task);
            }
            awaiting_diagram = None;
        }
    }
    if let Some(d) = detail.take() {
        doc.details.push(d);
    }

    // Requirement ids come from the detail sections (TSK-03).
    for phase in &mut doc.phases {
        for task in &mut phase.tasks {
            let Some(field) = doc
                .details
                .iter()
                .find(|d| d.id == task.id)
                .and_then(|d| d.field("Requirement"))
            else {
                continue;
            };
            task.requirements = match id_pattern {
                Some(re) => super::ids::ids_in_text(&field.text, re),
                None => field
                    .text
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect(),
            };
        }
    }
    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"# Tarefas

### Fase 1: Fundações — acme_gateway

```
T1 → T2 → T3
```

- [x] T1: Tabela de tokens com hash (a1b2c3d)
- [x] T2: Repositório (Linux e Windows) (b2c3d4e)
- [ ] T3: Migração

### Fase 2: Correções

- [x] T9: Corrigir TTL (f6a7b8c)
- [X] T7: Teste de carga
- [ ] T4 (aguardando CI): CI no GitHub (cfd7d06)

## Detalhamento das tarefas

```
- [x] T99: inside a code block, not a task
```

### T1: Tabela de tokens com hash

**What**: Criar a tabela.
**Onde**: `src/db/tokens.sql`
**Depende de**: nenhuma
**Requirement**: TOK-02, contrato (suporta TOK-01)
**Pronto quando**:

- Tabela criada
- Índice único

**Testes**: unit
**Gate**: fase

---

### T2: Repositório

**Depende de**: T1
**Requirement**: TOK-02
"#;

    fn re() -> Regex {
        Regex::new(r"[A-Z]{2,5}-\d{2}").unwrap()
    }

    #[test]
    fn phases_tasks_and_diagram() {
        let doc = tasks(DOC, Some(&re()));
        assert_eq!(doc.phases.len(), 2);
        let f1 = &doc.phases[0];
        assert_eq!(
            (f1.number, f1.title.as_str(), f1.repo.as_deref()),
            (Some(1), "Fundações", Some("acme_gateway"))
        );
        assert_eq!(
            f1.diagram,
            vec![vec!["T1".to_owned(), "T2".into(), "T3".into()]]
        );
        assert_eq!(
            (f1.done(), f1.total(), f1.state()),
            (2, 3, PhaseState::InProgress)
        );
        assert_eq!(f1.tasks[1].title, "Repositório (Linux e Windows)");
        assert_eq!(f1.tasks[1].hash.as_deref(), Some("b2c3d4e"));
        assert_eq!(f1.tasks[2].hash, None);
        let f2 = &doc.phases[1];
        assert!(f2.diagram.is_empty());
        let ids: Vec<&str> = f2.tasks.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["T9", "T7", "T4"]);
        assert!(f2.tasks[1].done);
        assert_eq!(f2.tasks[2].note.as_deref(), Some("aguardando CI"));
        assert_eq!(f2.tasks[2].hash.as_deref(), Some("cfd7d06"));
        assert_eq!(doc.progress(), (4, 6));
        assert!(doc.task("T99").is_none());
    }

    #[test]
    fn details_and_requirements() {
        let doc = tasks(DOC, Some(&re()));
        let d = doc.detail("T1").unwrap();
        assert_eq!(d.field("onde").unwrap().text, "`src/db/tokens.sql`");
        assert_eq!(
            d.field("Pronto quando").unwrap().items,
            ["Tabela criada", "Índice único"]
        );
        assert_eq!(d.field("Gate").unwrap().text, "fase");
        assert!(d.depends_on().is_empty());
        assert_eq!(doc.detail("T2").unwrap().depends_on(), ["T1"]);
        assert_eq!(doc.task("T1").unwrap().requirements, ["TOK-02", "TOK-01"]);
        let plain = tasks(DOC, None);
        assert_eq!(
            plain.task("T1").unwrap().requirements,
            ["TOK-02", "contrato (suporta TOK-01)"]
        );
    }

    #[test]
    fn complete_and_pending_states() {
        let doc = tasks(
            "### Fase 1: A\n- [x] T1: a\n### Fase 2: B\n- [ ] T2: b\n### Fase 3: C\n",
            None,
        );
        let states: Vec<PhaseState> = doc.phases.iter().map(Phase::state).collect();
        assert_eq!(
            states,
            [
                PhaseState::Complete,
                PhaseState::Pending,
                PhaseState::Pending
            ]
        );
    }

    #[test]
    fn tasks_without_phase_go_to_an_implicit_phase() {
        let doc = tasks(
            "- [x] T1: a\n- [ ] T2: b\n### Fase 1: X\n- [ ] T3: c\n",
            None,
        );
        assert_eq!(doc.phases.len(), 2);
        assert_eq!(doc.phases[0].number, None);
        assert_eq!(doc.phases[0].total(), 2);
        assert_eq!(doc.progress(), (1, 3));
    }

    #[test]
    fn out_of_format_is_empty() {
        assert!(tasks("", None).is_empty());
        assert!(tasks("# Just a title\n\nSome prose with - [x] inline.\n", None).is_empty());
        assert!(tasks("- [x] no id here\n- [ ] Tx: bad id\n", None).is_empty());
    }
}
