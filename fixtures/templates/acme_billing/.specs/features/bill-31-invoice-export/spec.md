# BILL-31 — Exportação de faturas em CSV

**Status:** Em validação · 2 de 2 fases · 4/4 tarefas
**Issue:** [BILL-31](https://tracker.example/browse/BILL-31)
**Idioma:** pt-BR

## Problema

O financeiro copia faturas à mão para planilhas.

## Histórias de usuário

### P1: Exportar ⭐ MVP

**Critérios de aceite**:

1. QUANDO o usuário exporta um mês ENTÃO o sistema DEVE gerar um CSV com uma linha por fatura. <!-- INV-01 -->
2. QUANDO uma fatura tem mais de 1000 itens ENTÃO o sistema DEVE paginar a geração. <!-- INV-02 -->
