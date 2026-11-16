# ACME-142 — Validação

## Validação: tokens de serviço - PASS ✅

| ID | Resultado da spec | Evidência | Status |
| --- | --- | --- | --- |
| TOK-01 | Valor exibido uma vez | `src/tokens/api.ts:40` | ✅ |
| TOK-02 | Só o hash é persistido | `src/tokens/repo.ts:12` | ✅ |
| TOK-03 | Recusa em até 5 s | `tests/load/revoke.ts:8` | ✅ |
| TOK-04 | Último uso registrado | — | ⏳ |

## Sensor de discriminação

| Local | Mutação | Critério | Resultado |
| --- | --- | --- | --- |
| `src/gateway/cache.ts:12` | TTL 60 s | TOK-03 | morto ✅ |

## Casos de borda

- Revogação durante requisição longa: coberto.

## Gates

- fase: 5 execuções, 1 falha corrigida (T9)

## Não verificado

- Carga acima de 1000 tokens.
