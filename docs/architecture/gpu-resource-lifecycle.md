# Ciclo de vida de recursos GPU

**Decisión:** los recursos GPU son caches derivados y descartables. Su ciclo de vida debe estar gobernado por invalidaciones explícitas, presupuestos de memoria y reconstrucción determinística.

## Estados

```text
Unallocated → Allocated → Dirty → Uploaded → Resident → Evicted/Rebuilt
```

## Recursos

| Recurso | Owner | Invalidación típica |
|---------|-------|---------------------|
| Vertex/Index Buffer | `buffers/` | `GeometryAdded`, `GeometryUpdated`, `GeometryDeleted` |
| Uniform Buffer | `camera/`, `pipeline/` | `CameraChanged`, `ThemeChanged` |
| Bind Group | `pipeline/` | cambio de layout/material/global uniforms |
| Pipeline | `pipeline/` | cambio de shader o configuración global |
| Preview Buffer | `scene/preview` | `PreviewChanged` |

## Reglas duras

1. No crear buffers nuevos por frame salvo recursos efímeros explícitos y acotados.
2. Preferir actualizaciones parciales (`dirty ranges`) para geometría persistente.
3. Separar buffers persistentes de buffers de preview/interacción.
4. Medir antes de optimizar: todo cambio de batching debe exponer diagnóstico básico.
5. La pérdida de dispositivo o recreación de swapchain debe reconstruir desde estado derivado, no desde UI.

## Checklist

- [ ] ¿El recurso tiene owner claro?
- [ ] ¿Se libera o reutiliza?
- [ ] ¿Está separado lo persistente de lo efímero?
- [ ] ¿Puede reconstruirse tras device lost/resync?
