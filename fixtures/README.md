# Fixtures

Synthetic workspace for Quire's tests and demos. **Everything here is invented**: no real
company, project, path or person. Project names use the fictional `acme` prefix and the
tracker URL is `tracker.example`.

`templates/` holds the versioned file contents. Git does not version nested repositories, so
`scripts/make-fixtures.sh` builds the actual workspace (git repos, branches and worktrees):

```bash
scripts/make-fixtures.sh                 # -> fixtures/workspace (gitignored)
scripts/make-fixtures.sh /tmp/quire-ws   # any empty or previously generated directory
```

The script is idempotent: it deletes and rebuilds the output directory, but only if that
directory is empty or carries the `.quire-fixtures` marker. Commits use a fixed author and
dates, and file mtimes are fixed, so commit hashes and "most recently modified" are
deterministic.

## Layout

```
<out>/
├─ projects/                         # scan path
│  ├─ acme_gateway/                  # repo, origin git@git.example.com:acme/acme_gateway.git, branch main
│  │  ├─ node_modules/…/.specs/      # decoy (gitignored)
│  │  └─ target/…/.specs/            # decoy (gitignored)
│  ├─ acme_billing/                  # repo without origin, branch main
│  │  └─ dist/…/.specs/              # decoy (gitignored)
│  └─ .worktrees/acme-142/acme_gateway/   # worktree, branch feat/acme-142-service-tokens, identical content
└─ external/acme-170/acme_gateway/   # worktree OUTSIDE the scan path, branch feat/acme-170-rate-limit, divergent
```

## Coverage

| Case | Where |
| --- | --- |
| Complete spec (7 files) + other files (`notes.md`, `payload.json`) | `acme_gateway/.specs/features/acme-142-service-tokens` |
| Spec identical across instances (same mtime: tie broken by path) | `acme-142-service-tokens`: 3 identical copies (main checkout and both worktrees; each worktree checks out the whole repo) |
| Spec divergent between instances (newer copy in the worktree) | `acme-170-rate-limit`: 3 copies, `tasks.md` differs (and is newer) in `external/acme-170` |
| Worktree outside the scan path (DSC-03) | `external/acme-170/acme_gateway` |
| Partial specs | `acme-170-rate-limit` (no summary/validation/JSONL), `acme-200-shared-auth`, `bill-40-refunds` |
| JSONL with an invalid line and a truncated last line | `acme_billing/.specs/features/bill-31-invoice-export/gates.jsonl` |
| JSONL with unknown extra fields | `bill-31-invoice-export` (`gates.jsonl`, `lessons.jsonl`) |
| Repo without `origin` | `acme_billing` |
| Same issue key in two projects (CLI-02) | `acme-200-shared-auth` in both repos |
| `.specs` inside `node_modules`, `target`, `dist` (must be ignored) | decoys created by the script |
| Project decisions file | `acme_gateway/.specs/STATE.md` |
| Tasks with out-of-order ids, phase without diagram, task without hash | `acme-142-service-tokens/tasks.md` (Fase 3) |
