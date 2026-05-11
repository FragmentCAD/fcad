# Modelo de SpatialIndex, snap y hit-test

**Decisión:** `SpatialIndex`, snap y hit-test son derivados del ECS. Son aceleradores de consulta, no fuentes de verdad del documento.

## Riesgo que resuelve

Se detectó una deriva crítica: el renderer podía sincronizar entidades desde `World`, mientras erase/hit-test/osnap consultaban un índice en `AppState` potencialmente vacío o stale. En CAD eso destruye confianza: el usuario ve una entidad, pero no puede seleccionarla o borrarla correctamente.

## Modelo

```text
ECS Geometry + Layer/Visibility
→ SpatialIndexSyncSystem
→ SpatialIndexSnapshot
→ SnapQuery / HitTestQuery
→ Tool Preview / Selection / Erase
```

## Tipos de consulta

| Consulta | Consume | Produce |
|----------|---------|---------|
| Hit-test | viewport point, tolerance, filters | entity refs + distance/order |
| Snap | cursor point, modes, filters | snap candidates + priority |
| Selection window | bounds, filters | ordered entity refs |
| Erase target | entity refs from hit-test/selection | `EraseEntity` command input |

## Reglas duras

1. `SpatialIndex` se reconstruye o actualiza desde ECS, nunca desde eventos UI sueltos.
2. Snap/hit-test devuelven referencias a entidades del Core, no geometría duplicada autoritativa.
3. Cambios de geometría, capa o visibilidad deben invalidar/sincronizar el índice.
4. El índice usado por herramientas debe corresponder al mismo snapshot/version del documento que ve el renderer.
5. Si el índice está stale, la respuesta correcta es resync/rebuild, no adivinar en Studio.

## Versionado mínimo

```text
DocumentVersion
SpatialIndexVersion
RenderSceneVersion
```

Las herramientas interactivas deben poder detectar divergencias y pedir resync.

## Checklist

- [ ] ¿El índice deriva de ECS?
- [ ] ¿La consulta devuelve `EntityId`/refs, no verdad duplicada?
- [ ] ¿La invalidación de índice acompaña cambios de geometría/capa/visibilidad?
- [ ] ¿Renderer, snap y hit-test operan sobre versiones compatibles?
