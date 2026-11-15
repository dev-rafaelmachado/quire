# ACME-170 — Limite de taxa por cliente

**Status:** Em andamento · 1 de 2 fases · 1/3 tarefas
**Issue:** [ACME-170](https://tracker.example/browse/ACME-170)
**Idioma:** pt-BR

## Problema

Um cliente com bug consegue derrubar o gateway com rajadas de requisições.

## Histórias de usuário

### P1: Limitar rajadas ⭐ MVP

**Critérios de aceite**:

1. QUANDO um cliente passa de 100 req/s ENTÃO o sistema DEVE responder 429. <!-- RTL-01 -->
2. QUANDO o limite é atingido ENTÃO o sistema DEVE informar `Retry-After`. <!-- RTL-02 -->

Relacionado: ACME-142 (tokens identificam o cliente).
