# Modelo runtime de CadEngine

**Decisión:** el runtime interno debe concentrar la coordinación del documento en un `CadEngine` explícito. `AppState` de Tauri solo hospeda servicios/adaptadores; no debe convertirse en una bolsa de estados CAD paralelos.

## Riesgo que resuelve

El análisis detectó riesgo P0 de race conditions, lock contention y estado repartido entre `World`, `SpatialIndex`, renderer y comandos Tauri. Si cada adapter toma locks o mantiene caches por su cuenta, Studio puede ver una cosa, snap otra y Core otra.

## Modelo

```text
Tauri AppState
→ CadEngineHandle
→ CadEngine
   ├── DocumentSession
   ├── World ECS
   ├── CommandBus
   ├── SpatialIndex sync
   ├── DomainEventLog
   ├── RenderInvalidationQueue
   └── Snapshot service
```

## Responsabilidades

| Pieza | Responsabilidad | No debe hacer |
|-------|-----------------|---------------|
| `AppState` | Exponer handles a Tauri, configuración nativa, servicios OS | Guardar verdad CAD paralela |
| `CadEngine` | Coordinar comandos, ECS, eventos, índices e invalidaciones | Depender de UI o WGPU |
| `CommandBus` | Validar y ejecutar `DomainCommand`s | Ser bypass de Tauri o MCP |
| `Snapshot service` | Entregar vistas compactas y consistentes | Exponer locks largos del `World` |
| `EventLog/Queues` | Ordenar consecuencias de dominio/render | Reemplazar al ECS como verdad |

## Reglas duras

1. Las mutaciones CAD pasan por `CadEngine.dispatch(command)` o equivalente.
2. `AppState` no almacena `SpatialIndex`, selección, geometría o documento como verdad independiente.
3. El renderer y Studio consumen snapshots/eventos; no sostienen locks largos sobre `World`.
4. Un comando debe producir eventos e invalidaciones en la misma transacción lógica.
5. Las colas de eventos/invalidation deben preservar orden por documento y comando.
6. Si se usa `RwLock<CadEngine>`, los guards deben ser cortos y nunca abarcar render frame, IO lento o llamadas LLM.

## Patrón de acceso

```text
Adapter request
→ acquire short engine access
→ dispatch command / query snapshot
→ release lock
→ stream events or response DTO
```

## Checklist

- [ ] ¿La operación toca el documento solo vía `CadEngine`?
- [ ] ¿El lock es corto y no cruza IO/render/LLM?
- [ ] ¿SpatialIndex e invalidaciones se actualizan como consecuencia del comando?
- [ ] ¿Existe snapshot para consumidores externos?
