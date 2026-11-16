// T2 spike (disposable): deterministic synthetic markdown, ~40 KB per unit of scale.
// Content is invented ("acme" fictional product); no real company data.

const LANGS = ["ts", "rust", "json", "bash", "sql", "toml"] as const;

const MERMAID: readonly string[] = [
  `flowchart TD
    A[Scan paths] --> B{Tem .specs?}
    B -- sim --> C[Registrar spec]
    B -- não --> D[Ignorar]
    C --> E[Resolver projeto via git]
    E --> F[(Índice SQLite)]
    F --> G[Sidebar]
    D --> H[Próximo diretório]
    H --> B`,
  `sequenceDiagram
    participant U as Usuário
    participant UI as Webview
    participant S as Shell
    participant C as Core
    U->>UI: abre spec
    UI->>S: get_spec(project, folder)
    S->>C: consulta índice
    C-->>S: SpecDetail
    S-->>UI: JSON
    UI-->>U: abas renderizadas
    C--)UI: spec://changed`,
  `stateDiagram-v2
    [*] --> Pendente
    Pendente --> EmAndamento: agente começa
    EmAndamento --> Bloqueada: dependência falhou
    Bloqueada --> EmAndamento: desbloqueio
    EmAndamento --> Concluida: gate verde
    Concluida --> [*]`,
  `classDiagram
    class Project {
      +String name
      +String gitCommonDir
      +Preset preset
    }
    class Spec {
      +String folder
      +String issueKey
    }
    class Instance {
      +String path
      +String branch
      +u64 mtime
    }
    Project "1" --> "*" Spec
    Spec "1" --> "*" Instance`,
  `erDiagram
    PROJECTS ||--o{ SPECS : contem
    SPECS ||--o{ INSTANCES : tem
    INSTANCES ||--o{ FILES : lista
    PROJECTS {
      int id
      string name
    }
    SPECS {
      int id
      string folder
    }`,
  `gantt
    title Plano sintético
    dateFormat YYYY-MM-DD
    section Fase 0
    Scaffold      :done, t1, 2026-01-05, 2d
    Spike webview :active, t2, after t1, 3d
    Spike watcher :t3, after t2, 3d
    section Fase 1
    Fixtures      :t5, after t3, 2d
    Scanner       :t6, after t5, 4d`,
  // Intentionally invalid, to exercise the per-block error path (VIW-03).
  `flowchart TD
    A[Início] --> B{{quebrado
    B -->> C[[[ sem fechamento`,
];

const WORDS =
  "spec tarefa fase gate índice worktree projeto preset leitura cache watcher evento commit branch revisão agente sumário validação lição requisito critério dependência fluxo arquivo pasta"
    .split(" ");

function sentence(seed: number, len: number): string {
  const out: string[] = [];
  for (let i = 0; i < len; i++) {
    out.push(WORDS[(seed * 7 + i * 13) % WORDS.length] ?? "spec");
  }
  const s = out.join(" ");
  return s.charAt(0).toUpperCase() + s.slice(1) + ".";
}

function paragraph(seed: number): string {
  return [0, 1, 2, 3, 4].map((k) => sentence(seed + k, 9 + ((seed + k) % 6))).join(" ");
}

function table(seed: number): string {
  const head = "| ID | Tarefa | Estado | Commit | Duração |\n| --- | --- | --- | --- | ---: |";
  const rows: string[] = [];
  for (let r = 0; r < 7; r++) {
    const n = seed * 10 + r;
    const state = ["concluída", "em andamento", "pendente"][n % 3] ?? "pendente";
    rows.push(
      `| DSC-${String(n % 100).padStart(2, "0")} | ${sentence(n, 4).slice(0, -1)} | ${state} | \`${(n * 2654435761).toString(16).slice(0, 7)}\` | ${(n % 90) + 3}s |`,
    );
  }
  return `${head}\n${rows.join("\n")}`;
}

function tasks(seed: number): string {
  const items: string[] = [];
  for (let t = 0; t < 6; t++) {
    const n = seed * 6 + t;
    items.push(`- [${n % 3 === 0 ? " " : "x"}] T${n}: ${sentence(n, 6).slice(0, -1)} <!-- TSK-${String(n % 100).padStart(2, "0")} -->`);
  }
  return items.join("\n");
}

function code(lang: (typeof LANGS)[number], seed: number): string {
  switch (lang) {
    case "ts":
      return `\`\`\`ts
export interface SpecSummary {
  projectId: number;
  folder: string;
  progress: { done: number; total: number };
  instanceCount: number;
}

export async function loadSpecs(filter: string): Promise<SpecSummary[]> {
  const all = await invoke<SpecSummary[]>("list_projects", { filter });
  return all
    .filter((s) => s.folder.includes(filter))
    .sort((a, b) => a.folder.localeCompare(b.folder));
}

const threshold = ${seed * 3};
for (const spec of await loadSpecs("acme")) {
  if (spec.progress.done > threshold) console.log(spec.folder);
}
\`\`\``;
    case "rust":
      return `\`\`\`rust
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("path not found: {0}")]
    NotFound(PathBuf),
    #[error("permission denied: {0}")]
    Denied(PathBuf),
}

pub fn scan(root: &Path) -> Result<Vec<Discovery>, ScanError> {
    let mut out = Vec::with_capacity(${seed + 16});
    for entry in WalkBuilder::new(root).hidden(false).build() {
        let entry = entry.map_err(|_| ScanError::Denied(root.into()))?;
        if entry.file_name() == ".specs" {
            out.push(Discovery::from(entry.path()));
        }
    }
    Ok(out)
}
\`\`\``;
    case "json":
      return `\`\`\`json
{
  "ts": "2026-01-${String((seed % 28) + 1).padStart(2, "0")}T10:00:00Z",
  "level": "phase",
  "status": "${seed % 4 === 0 ? "fail" : "pass"}",
  "head": "${(seed * 40503).toString(16).padStart(7, "0").slice(0, 7)}",
  "duration_ms": ${seed * 137 + 900},
  "cmd": "cargo test --workspace",
  "tail": ["running 12 tests", "test result: ok. 12 passed"]
}
\`\`\``;
    case "bash":
      return `\`\`\`bash
#!/usr/bin/env bash
set -euo pipefail
for repo in ~/acme/*/; do
  if [ -d "$repo/.specs" ]; then
    git -C "$repo" worktree list --porcelain | awk '/^worktree/ {print $2}'
  fi
done | sort -u > /tmp/worktrees-${seed}.txt
wc -l /tmp/worktrees-${seed}.txt
\`\`\``;
    case "sql":
      return `\`\`\`sql
SELECT p.name, s.folder, COUNT(i.id) AS instances
FROM projects p
JOIN specs s ON s.project_id = p.id
LEFT JOIN instances i ON i.spec_id = s.id
WHERE p.preset = 'generic'
GROUP BY p.name, s.folder
HAVING COUNT(i.id) > ${seed % 3}
ORDER BY instances DESC, s.folder;
\`\`\``;
    case "toml":
      return `\`\`\`toml
scan_paths = ["~/acme", "~/sandbox"]
poll_interval_ms = ${3000 + seed}

[projects."acme_gateway"]
preset = "generic"
alias = "gateway-${seed}"
issue_url_base = "https://tracker.example/browse/"
\`\`\``;
  }
}

const ASCII = `\`\`\`
┌──────────┐    ┌──────────┐    ┌──────────┐
│  scan    │───▶│  índice  │───▶│    UI    │
└──────────┘    └──────────┘    └──────────┘
      ▲               │
      │  eventos      ▼
┌──────────┐    ┌──────────┐
│ watcher  │◀───│  disco   │
└──────────┘    └──────────┘
\`\`\``;

function section(i: number): string {
  const parts: string[] = [];
  parts.push(`## ${i + 1}. ${sentence(i, 4).slice(0, -1)}`);
  parts.push(paragraph(i));
  parts.push(`### ${i + 1}.1 Tabela de acompanhamento`);
  parts.push(table(i));
  parts.push(paragraph(i + 50));
  parts.push(`### ${i + 1}.2 Tarefas`);
  parts.push(tasks(i));
  parts.push(`### ${i + 1}.3 Código`);
  parts.push(code(LANGS[i % LANGS.length] ?? "ts", i));
  parts.push(paragraph(i + 100));
  if (i % 10 < MERMAID.length) {
    parts.push(`### ${i + 1}.4 Diagrama`);
    parts.push(`\`\`\`mermaid\n${MERMAID[i % 10]}\n\`\`\``);
  }
  if (i % 3 === 1) {
    parts.push(ASCII);
  }
  parts.push(`> **Nota:** ${sentence(i + 7, 14)} Veja [o design](./design.md#${i}) e a issue \`ACME-${1000 + i}\`.`);
  parts.push(code(LANGS[(i + 3) % LANGS.length] ?? "ts", i + 1));
  parts.push(paragraph(i + 150));
  return parts.join("\n\n");
}

export function generateMarkdown(scale: number): string {
  const sections = Math.max(1, Math.round(10 * scale));
  const head = `# ACME-1452 — Spec sintética para o spike\n\n**Status:** rascunho\n**Issue:** ACME-1452\n**Idioma:** pt-BR\n\n${paragraph(999)}`;
  const body: string[] = [];
  for (let i = 0; i < sections; i++) body.push(section(i));
  return `${head}\n\n${body.join("\n\n")}\n`;
}
