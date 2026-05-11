# Modelo de estado derivado del Renderer

**Decisión:** `fcad-renderer` representa consecuencias gráficas del dominio. Puede cachear agresivamente para rendimiento, pero nunca decide qué existe en el documento CAD.

## Modelo

```text
ECS Domain State
→ DomainEvent
→ RenderInvalidation
→ RenderScene derivada
→ GPU Buffers / Pipelines
→ Frame
```

## Responsabilidades

| Área | Puede hacer | No puede hacer |
|------|-------------|----------------|
| RenderScene | Mantener representación dibujable derivada | Ser fuente de verdad CAD |
| Tessellation | Convertir geometría analítica a mallas | Validar reglas arquitectónicas |
| Buffers | Gestionar VRAM, dirty ranges, batching | Decidir existencia de entidades |
| Camera | Proyección, pan/zoom, uniforms | Mutar documento |
| Diagnostics | FPS, GPU stats, overlays debug | Cambiar dominio para “corregir” vista |

## Estructura recomendada

```text
fcad-renderer/src/
├── scene/              # RenderScene derivada
├── invalidation/       # mapping invalidación -> dirty work
├── buffers/            # lifecycle de buffers GPU
├── pipeline/           # WGPU pipelines y bind groups
├── tessellation/       # geometría analítica -> render mesh
├── shaders/            # WGSL
├── camera/             # vista/proyección/uniforms
└── diagnostics/        # métricas y debug overlays
```

## Reglas duras

1. Renderer consume `RenderInvalidation`; no hace polling semántico del ECS.
2. Caches GPU y dirty ranges deben poder reconstruirse desde snapshot + eventos.
3. Previews, hover y selección visual viven en capas efímeras separadas de entidades persistidas.
4. El renderer no interpreta reglas de capas, snap, constraints ni compliance.
5. `f32` es aceptable para GPU, pero la conversión desde coordenadas de dominio debe preservar estabilidad visual.
6. Locks largos por frame están prohibidos; preferir snapshots compactos y colas de invalidación.

## Capas de render

| Capa | Contenido | Persistencia |
|------|-----------|--------------|
| Document | entidades confirmadas | Derivada del Core |
| Annotation | textos, cotas, helpers | Derivada del Core/config |
| Interaction | hover, selección, grips | Efímera |
| Preview | rubber-band, AI preview, tool preview | Efímera |
| Debug | overlays, bounds, stats | Efímera |

## Checklist

- [ ] ¿El cambio visual viene de una invalidación explícita?
- [ ] ¿La cache GPU puede reconstruirse?
- [ ] ¿Preview y documento están separados?
- [ ] ¿La lógica de dominio sigue fuera del renderer?
