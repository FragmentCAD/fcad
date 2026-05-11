# Contrato de Invalidación Render

**Decisión:** el renderer no debe descubrir cambios por intuición ni por polling semántico. El Core emite invalidaciones explícitas y el renderer actualiza solo las partes necesarias.

## Principio

`fcad-renderer` puede tener caches GPU, buffers, dirty ranges y estado gráfico derivado. Eso mejora rendimiento. Pero esos datos no son verdad del documento: son una proyección del ECS.

## Eventos mínimos

| Invalidación | Causa típica | Acción esperada |
|--------------|--------------|-----------------|
| `GeometryAdded` | Nueva entidad con geometría | Agregar vértices/batch correspondiente. |
| `GeometryUpdated` | Edición de entidad existente | Marcar rango/batch dirty. |
| `GeometryDeleted` | Borrado/tombstone | Remover u ocultar representación GPU. |
| `LayerVisibilityChanged` | Capa visible/oculta | Refiltrar batches o uniforms. |
| `CameraChanged` | Pan/zoom/orbit 2D | Actualizar uniforms de cámara. |
| `ThemeChanged` | Cambio de tema | Actualizar uniforms/colores derivados. |
| `PreviewChanged` | Rubber band, AI preview, hover | Actualizar buffers efímeros. |

## Reglas duras

1. La cámara puede estar modelada en Core, pero el renderer solo consume su estado derivado.
2. Los previews son efímeros: no son entidades persistidas hasta que un comando los commitea.
3. El renderer no debe consultar reglas de capa, snap o constraints para decidir dominio.
4. Cada invalidación debe ser idempotente o tener orden transaccional claro.
5. Si se usa `Mutex<World>`, hay que evitar locks largos por frame; preferir snapshots/eventos compactos cuando el sistema crezca.

## Riesgo que evita

Sin este contrato aparecen bugs CAD graves:

- el usuario ve una cosa y snap detecta otra;
- erase borra una entidad que ya no coincide con pantalla;
- undo cambia el dominio pero no refresca GPU;
- IA genera preview que parece commiteado.

## Checklist de implementación

- [ ] ¿Cada mutación de geometría produce invalidación?
- [ ] ¿Los previews están separados de entidades persistidas?
- [ ] ¿La GPU cache puede reconstruirse desde ECS + eventos?
- [ ] ¿La UI nunca fuerza un redraw como sustituto de una invalidación faltante?
