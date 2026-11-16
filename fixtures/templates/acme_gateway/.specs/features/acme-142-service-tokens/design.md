# ACME-142 — Design

**Spec:** `spec.md`

## Visão geral

```mermaid
graph TD
    A[Admin] --> B[API de tokens]
    B --> C[(Tabela tokens: hash)]
    D[Integração] --> E[Gateway]
    E --> C
```

## Fluxo de revogação

```
admin ──revoga──▶ API ──invalida──▶ cache (TTL 5 s)
                              │
                              └──▶ tabela tokens
```

## Decisões

- **TD-01:** hash com SHA-256 e sal por token (ver AD-001).
