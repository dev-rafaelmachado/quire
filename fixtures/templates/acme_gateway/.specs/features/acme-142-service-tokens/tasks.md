# ACME-142 — Tarefas

## Plano de execução

### Fase 1: Fundações — acme_gateway

```
T1 → T2 → T3
```

- [x] T1: Tabela de tokens com hash (a1b2c3d)
- [x] T2: Repositório de tokens (b2c3d4e)
- [x] T3: Migração e seed sintético (c3d4e5f)

### Fase 2: Emissão e revogação — acme_gateway

```
T4 → T5
T4 → T6
```

- [x] T4: Endpoint de emissão (d4e5f6a)
- [x] T5: Revogação com invalidação de cache (e5f6a7b)
- [ ] T6: Registro de último uso

### Fase 3: Correções

- [x] T9: Corrigir TTL do cache de revogação (f6a7b8c)
- [x] T7: Teste de carga da revogação

## Detalhamento das tarefas

### T1: Tabela de tokens com hash

**What**: Criar a tabela `tokens` guardando só o hash.
**Onde**: `src/db/tokens.sql`
**Depende de**: nenhuma
**Requirement**: TOK-02
**Pronto quando**:

- Tabela criada com coluna `hash` e índice único

**Testes**: unit
**Gate**: fase

---

### T2: Repositório de tokens

**What**: Funções de criar, buscar por hash e revogar.
**Onde**: `src/tokens/repo.ts`
**Depende de**: T1
**Requirement**: TOK-02, TOK-03
**Pronto quando**:

- Busca por hash retorna o token ativo
- Token revogado não é retornado

**Testes**: unit
**Gate**: fase

---

### T3: Migração e seed sintético

**What**: Migração idempotente e seed com dados falsos.
**Onde**: `src/db/migrations/`
**Depende de**: T1
**Requirement**: TOK-02
**Pronto quando**:

- Rodar a migração duas vezes não falha

**Testes**: integration
**Gate**: fase

---

### T4: Endpoint de emissão

**What**: `POST /tokens` devolve o valor uma única vez.
**Onde**: `src/tokens/api.ts`
**Depende de**: T2
**Requirement**: TOK-01, TOK-02
**Pronto quando**:

- O valor do token aparece só na resposta da criação
- A tabela guarda apenas o hash

**Testes**: unit, integration
**Gate**: fase

---

### T5: Revogação com invalidação de cache

**What**: `DELETE /tokens/:id` invalida o cache do gateway.
**Onde**: `src/tokens/api.ts`, `src/gateway/cache.ts`
**Depende de**: T4
**Requirement**: TOK-03
**Pronto quando**:

- Chamadas com o token revogado falham em até 5 s

**Testes**: integration
**Gate**: fase

---

### T6: Registro de último uso

**What**: Atualizar `last_used_at` de forma assíncrona.
**Onde**: `src/gateway/audit.ts`
**Depende de**: T4
**Requirement**: TOK-04
**Pronto quando**:

- `last_used_at` atualizado sem aumentar a latência p95

**Testes**: integration
**Gate**: fase

---

### T9: Corrigir TTL do cache de revogação

**What**: O TTL estava em 60 s; reduzir para 5 s.
**Onde**: `src/gateway/cache.ts:12`
**Depende de**: T5
**Requirement**: TOK-03
**Pronto quando**:

- Teste de revogação passa com 5 s

**Testes**: integration
**Gate**: fase

---

### T7: Teste de carga da revogação

**What**: Medir a revogação com 1000 tokens ativos.
**Onde**: `tests/load/revoke.ts`
**Depende de**: T9
**Requirement**: TOK-03
**Pronto quando**:

- p95 da revogação abaixo de 5 s

**Testes**: e2e
**Gate**: final
