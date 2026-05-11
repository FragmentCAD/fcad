# AI Runtime Bridge

**Decisión:** la IA nativa de FragmentCAD debe operar sobre intenciones y planes verificables, no sobre geometría cruda sin control. MCP se mantiene como protocolo externo; el runtime interno debe ser el command bus del Core.

## Modelo recomendado

```text
AI Console / Agent / MCP
→ AI Intent
→ DomainCommand candidate
→ Core validation
→ Preview
→ Human approval or policy approval
→ Commit
```

## Qué sí puede hacer la IA

| Acción IA | Forma correcta |
|-----------|----------------|
| Crear una habitación | `CreateRoomEnvelope` + validación de área/capas |
| Sugerir layout | `SuggestLayoutPlan` devuelve alternativas y conflictos |
| Insertar bloques | Plan referenciado a `fcad-assets` y capas estándar |
| Marcar área | Entidad semántica o preview, no geometría definitiva inmediata |
| Completar muros/aberturas | `ApplyLayoutPlan` luego de validación y preview |

## Qué no debe hacer

- No escribir directamente buffers del renderer.
- No mutar `World` saltando el command bus.
- No inventar capas, grosores o nombres si existen en `fcad-assets`.
- No convertir prompts en geometría persistente sin validación.
- No usar MCP como loop interactivo de alta frecuencia.

## Caso guía: marcar área y crear room

```text
Usuario marca polígono
→ CreateRoomEnvelopeCommand
→ Core crea RoomEnvelope semántico
→ IA propone LayoutPlan con rooms/walls/openings
→ Core valida constraints y estándares
→ Renderer muestra preview efímero
→ Usuario aprueba
→ ApplyLayoutPlanCommand materializa entidades ECS
```

## Guardrails mínimos

- [ ] Toda acción IA produce un plan explicable.
- [ ] El Core valida constraints antes de persistir.
- [ ] El usuario puede ver preview antes del commit.
- [ ] El resultado referencia estándares de `fcad-assets` cuando aplica.
- [ ] El flujo es testeable sin UI.
