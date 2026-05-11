# IPC Command Gateway y sincronización Tauri

**Decisión:** Tauri es un adaptador nativo, no el centro del dominio. Toda mutación CAD debe entrar por un gateway tipado que despacha `DomainCommand`s hacia `fcad-core`; la sincronización hacia Studio y Renderer sale como batches ordenados de eventos e invalidaciones.

## Modelo obligatorio

```text
Preact Tool/UI
→ typed command API
→ Tauri command: dispatch_domain_command
→ Core CommandBus
→ ECS Mutation
→ DomainEventBatch + RenderInvalidationBatch
→ Tauri Channel
→ Studio Signals + RendererBridge
```

## Contratos

| Contrato | Uso | Regla |
|----------|-----|-------|
| Command Gateway | Mutaciones CAD request/response | Único punto IPC para modificar documento. |
| Event Stream | Hechos de dominio y cambios de vista | Debe ser ordenado, batchado y versionado. |
| Snapshot Protocol | Apertura, resync, recovery, debug | Reconstruye estado derivado sin confiar en memoria UI. |
| Error Envelope | Errores técnicos y de dominio | No devolver strings sueltos desde Rust. |

## Tauri v2: uso recomendado

- Usar `#[tauri::command]` para requests explícitos: abrir, guardar, ejecutar comando, pedir preview o snapshot.
- Usar `tauri::ipc::Channel<T>` para streams ordenados: eventos del documento, progreso, IA, invalidaciones y diagnósticos.
- Usar `tauri::State<T>` para inyectar `AppState`, `DocumentSession`, `CommandBus` y `RenderBridge`.
- Usar plugins de Tauri solo para capacidades de sistema: filesystem, dialog, updater, clipboard, shortcuts.

## Reglas duras

1. Ningún componente Preact llama `invoke()` directo.
2. Ningún comando Tauri muta `World` por fuera del `CommandBus`.
3. Ningún evento global suelto reemplaza el stream tipado de dominio.
4. Cada mensaje cruzando IPC debe tener tipo, versión y correlación opcional (`command_id`, `document_id`).
5. Las operaciones largas deben emitir progreso por channel, no bloquear el event loop.
6. Ante desincronización, Studio debe pedir snapshot; no intentar “arreglar” estado local a mano.

## Decisión: no abandonar Tauri

FragmentCAD mantiene UI web con Preact y runtime nativo Rust porque esa división maximiza velocidad de iteración, accesibilidad, theming, layout complejo y rendimiento del core. Hacer toda la UI en Rust simplificaría el lenguaje, pero encarecería paneles, docking, command palette, extensibilidad y diseño visual.

## Checklist

- [ ] ¿La mutación entra por `dispatch_domain_command` o equivalente?
- [ ] ¿El payload está tipado en TS y Rust?
- [ ] ¿La respuesta separa éxito, error de dominio y error técnico?
- [ ] ¿Los eventos salen batchados y ordenados?
- [ ] ¿Existe camino de snapshot para resync?
