# Modelo del paquete fcad-ai

**Decisión:** `fcad-ai` será un cliente cognitivo del Core, no una extensión oculta del Core ni un runtime MCP. Recuerda, razona y propone; `fcad-core` valida, muta y emite verdad.

## Modelo de relación

```text
fcad-studio  → experiencia humana / UI
fcad-ai      → agente, memoria, planning, propuestas
fcad-core    → verdad CAD, validación, mutación ECS
fcad-renderer→ representación derivada
MCP/CLI      → adapters externos sobre capacidades internas
```

## Estructura objetivo

```text
fcad-ai/
├── agent/
│   ├── modes/
│   │   ├── assisted/
│   │   └── solo/
│   ├── workflows/
│   ├── tools/
│   └── responses/
├── memory/
├── planning/
├── core-client/
├── interaction/
├── policies/
├── audit/
├── adapters/
└── contracts/
```

## Responsabilidades

| Área | Hace | No hace |
|------|------|---------|
| `agent` | Orquesta modos, tools y respuestas | Guardar verdad CAD |
| `memory` | Observaciones, retrieval, summaries | Mutar Core |
| `planning` | Briefs, programas, layouts, alternativas | Aplicar cambios directos |
| `core-client` | Snapshots, queries, preview, validación | Validar por cuenta propia |
| `interaction` | Prompt, feedback, accept/reject/refine | Calcular dominio |
| `policies` | Autonomía, aprobación, riesgo | Saltar validación Core |
| `audit` | Trazabilidad y reportes | Ser fuente de verdad |
| `adapters` | MCP, CLI boosts, Studio bridge, Engram | Contener reglas principales |

## Reglas duras

1. `fcad-ai` no muta ECS directamente.
2. Toda acción materializable sale como `IntentPlan`, `DomainCommandProposal` o `CommitRequest` validable.
3. Studio puede usar `fcad-ai` para interacción agente/usuario, pero no implementa razonamiento de agente.
4. MCP expone capacidades de `fcad-ai`/Core hacia afuera; no define el runtime interno.
5. CLI boosts son adapters internos testeables, no bypass de políticas.

## Checklist

- [ ] ¿La lógica pertenece a agente, memoria, planning, policies o adapter?
- [ ] ¿La propuesta pasa por `core-client` y validación Core?
- [ ] ¿La interacción con Studio usa contratos tipados?
- [ ] ¿La decisión queda auditada?
