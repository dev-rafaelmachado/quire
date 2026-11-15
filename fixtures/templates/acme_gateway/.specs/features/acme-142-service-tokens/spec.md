# ACME-142 — Tokens de serviço para integrações

**Status:** Concluído, aguardando revisão · 3 de 3 fases · 7/8 tarefas · verificação full PASS
**Issue:** [ACME-142](https://tracker.example/browse/ACME-142)
**Idioma:** pt-BR

## Fontes

- Pedido do time de integrações (sintético)
- `src/main.ts:1` (handler atual)

## Problema

Integrações automáticas usam credenciais de usuários reais. Quando a pessoa sai do time, a integração quebra.

## Objetivos

- Emitir tokens de serviço independentes de usuário.
- Revogar um token sem afetar os demais.

## Fora do escopo

- Rotação automática de tokens.

## Premissas e perguntas em aberto

- **A1:** um cliente tem no máximo 20 tokens ativos.

## Histórias de usuário

### P1: Emitir token ⭐ MVP

**User Story**: Como administrador, quero emitir um token de serviço para que integrações não dependam de usuários.

**Critérios de aceite**:

1. QUANDO o administrador emite um token ENTÃO o sistema DEVE mostrar o valor uma única vez. <!-- TOK-01 -->
2. QUANDO o token é guardado ENTÃO o sistema DEVE persistir apenas o hash. <!-- TOK-02 -->

### P1: Revogar token ⭐ MVP

**User Story**: Como administrador, quero revogar um token para cortar um acesso indevido.

**Critérios de aceite**:

1. QUANDO um token é revogado ENTÃO o sistema DEVE recusar chamadas com ele em até 5 s. <!-- TOK-03 -->

### P2: Auditoria

**Critérios de aceite**:

1. QUANDO um token é usado ENTÃO o sistema DEVE registrar o último uso. <!-- TOK-04 -->

## Casos de borda

- Token revogado durante uma requisição longa: a requisição em curso termina.

## Rastreabilidade de requisitos

| Requisito | Tarefas |
| --- | --- |
| TOK-01, TOK-02 | T4 |
| TOK-03 | T5, T9 |
| TOK-04 | T6 |

## Critérios de sucesso

- Nenhuma integração usa credencial de usuário.
