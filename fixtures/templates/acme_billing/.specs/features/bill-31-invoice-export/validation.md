# BILL-31 — Validação

## Validação: exportação de faturas - FAIL ❌

| ID | Resultado da spec | Evidência | Status |
| --- | --- | --- | --- |
| INV-01 | Uma linha por fatura | `src/invoice.rs:1` | ✅ |
| INV-02 | Paginação acima de 1000 itens | — | ❌ |
