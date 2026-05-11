# Roadmap IA-first hacia v0 usable

**Decisión:** antes de sumar features vistosas, FragmentCAD debe estabilizar su columna vertebral: autoridad de estado, command bus, invalidación render y bridge IA. Ese es el camino para llegar a un CAD IA-first usable, no solo a una demo.

## Fases

| Fase | Objetivo | Criterio de salida |
|------|----------|--------------------|
| 0. Loop interactivo confiable | Unificar ECS como verdad, command bus e invalidación render | Snap, erase, hit-test y renderer coinciden siempre. |
| 1. CAD 2D mínimo usable | Herramientas base para trabajo real pequeño | Usuario crea/edita/guarda/importa/exporta un plano simple sin pérdida de estado. |
| 2. IA asistida práctica | IA ayuda en tareas concretas con preview y control humano | Crear ambiente desde área marcada y sugerir layout reduce trabajo manual. |
| 3. IA-first semántico | Entidades arquitectónicas y constraints reales | El sistema entiende rooms/walls/openings/zones y audita decisiones IA. |

## Fase 0: fundación obligatoria

Entregables:

- ECS como única autoridad.
- SpatialIndex como recurso derivado/sincronizado, no estado paralelo.
- Command bus interno tipado.
- `DomainEvent` + `RenderInvalidation` explícitos.
- MCP como adapter externo.
- AI Runtime Bridge inicial.

## Fase 1: CAD 2D mínimo usable

Entregables:

- Select, line, rect, erase.
- Layers activas y visibilidad.
- Grid, ortho, osnap confiables.
- Undo/redo transaccional.
- Persistencia `.fcad`.
- DXF import/export mínimo confiable.

## Fase 2: IA asistida práctica

Entregables:

- AI Console conectada a runtime real.
- Primer flujo: marcar área → crear ambiente.
- Segundo flujo: sugerir layout de muros/aberturas con preview.
- Guardrails: validación Core + aprobación humana.

## Fase 3: IA-first semántico

Entregables:

- Entidades `Room`, `Wall`, `Opening`, `Zone`.
- Constraints arquitectónicos.
- Reglas desde `fcad-assets`.
- Trazabilidad: qué propuso la IA, por qué y qué se aprobó.

## Regla de priorización

Si una feature nueva no fortalece el flujo:

```text
Intent → Command → ECS → Event → Invalidation → Render/Preview
```

entonces no pertenece a la Fase 0.
