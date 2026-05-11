# Workflow del agente y SOLO Mode

**Decisión:** SOLO es un modo de ejecución del agente, no un subsistema separado ni un bypass del Core. Reduce fricción humana entre pasos seguros, pero mantiene validación, políticas, auditoría y rollback.

## Modos

| Modo | Flujo | Uso |
|------|-------|-----|
| Assisted | Plan → Ask → Preview → Ask → Commit | Trabajo profesional con revisión frecuente |
| SoloDraft | Plan → Validate → Auto-commit reversible → Report | MVPs, bocetos, variantes rápidas |
| SoloStrict | Plan → Validate clean → Auto-commit → Report | Cambios automáticos solo sin warnings críticos |
| SoloExperimental | Plan → Validate → Commit with warnings → Report | Exploración controlada |

## Pipeline para proyecto desde cero

```text
UserPrompt
→ DesignBrief
→ Requirements / Constraints
→ RoomProgram
→ AdjacencyGraph
→ LayoutProposal(s)
→ Core Validation
→ Preview
→ Staged DomainCommands
→ Audit Report
```

## Reglas duras

1. El agente no genera geometría cruda persistente como salida final.
2. Las propuestas se materializan por comandos de dominio staged y auditables.
3. SOLO puede omitir aprobaciones humanas intermedias, pero no Core validation, policies, audit ni rollback.
4. Cada sesión SOLO debe registrar prompt, modo, pasos, validaciones, warnings, snapshots y comandos aplicados.
5. Si una política marca riesgo alto, SOLO debe detenerse o degradar a Assisted.

## Sesión SOLO

```text
SoloSession
├── id
├── mode
├── user_prompt
├── constraints
├── steps[]
├── snapshot_before
├── snapshot_after
└── report
```

## Checklist

- [ ] ¿SOLO se implementa bajo `agent/modes/solo`?
- [ ] ¿La autonomía está gobernada por `policies`?
- [ ] ¿Cada commit automático es reversible?
- [ ] ¿El usuario recibe reporte y explicación final?
