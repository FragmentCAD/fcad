# Command Bus y Eventos de Dominio

**Decisión:** todo cambio del documento debe pasar por un command bus interno tipado. Tauri, herramientas CAD, AI Console y MCP son adaptadores; ninguno debe mutar el documento por caminos propios.

## Modelo

```text
Adapter → InputIntent → DomainCommand → Handler/System → DomainEvent → RenderInvalidation
```

| Capa | Ejemplos | Regla |
|------|----------|-------|
| Adapter | Tauri IPC, MCP, AI Console, CLI | Traduce entrada externa a intención tipada. |
| InputIntent | Click, drag, prompt, hotkey | No muta el ECS directamente. |
| DomainCommand | `CreateLine`, `EraseEntity`, `ApplyLayoutPlan` | Es la unidad transaccional de cambio. |
| DomainEvent | `EntityCreated`, `EntityDeleted`, `LayerChanged` | Describe hechos ya aplicados. |
| RenderInvalidation | `GeometryAdded`, `PreviewChanged`, `CameraChanged` | Optimiza actualización del renderer. |

## Reglas duras

1. No crear comandos ad-hoc que muten `World` desde un componente UI.
2. No saltar el command bus para “hacerlo rápido”. Ese atajo rompe undo/redo, replay, auditoría e IA.
3. Un `DomainCommand` debe ser validable y testeable en Rust sin Tauri.
4. Si una mutación debe verse, debe producir invalidación explícita.
5. Si una operación IA genera múltiples cambios, debe agruparse como transacción.

## Comandos iniciales sugeridos

| Comando | Propósito |
|---------|-----------|
| `CreateLine` | Crear línea CAD básica. |
| `CreateRectangle` | Crear rectángulo o polyline cerrada. |
| `EraseEntity` | Borrado transaccional con soporte futuro para undo. |
| `SelectEntity` | Cambiar selección como estado controlado. |
| `ChangeActiveLayer` | Actualizar capa activa. |
| `CreateRoomEnvelope` | Crear área semántica para IA. |
| `ApplyLayoutPlan` | Materializar una propuesta IA validada. |

## Criterio de aceptación

- [ ] Toda mutación de documento es trazable a un `DomainCommand`.
- [ ] Cada comando declara eventos esperados.
- [ ] Undo/redo futuro puede reconstruirse desde comandos o deltas.
- [ ] MCP puede exponerse como adapter sin duplicar lógica.
