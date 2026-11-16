# ACME-170 — Tarefas

### Fase 1: Bucket — acme_gateway

```
T1 → T2
```

- [x] T1: Token bucket em memória (1a2b3c4)
- [ ] T2: Resposta 429 com Retry-After

### Fase 2: Observabilidade — acme_gateway

- [ ] T3: Métrica de rejeições

## Detalhamento das tarefas

### T1: Token bucket em memória

**What**: Bucket por cliente.
**Requirement**: RTL-01
**Gate**: fase

---

### T2: Resposta 429 com Retry-After

**What**: Responder 429 com o cabeçalho.
**Depende de**: T1
**Requirement**: RTL-01, RTL-02
**Gate**: fase

---

### T3: Métrica de rejeições

**What**: Contador por cliente.
**Depende de**: T2
**Requirement**: RTL-01
**Gate**: fase
