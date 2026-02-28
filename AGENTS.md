# 🤖 Agent Guidelines: fcad-renderer (The Eye)

Este documento define las reglas para el desarrollo del motor de renderizado de bajo nivel.

**Contexto:** Motor gráfico basado en WGPU y WGSL. Rendimiento crítico y precisión visual.

---

## 1. Arquitectura: Layered Pipeline (Data-Oriented)

Organización basada en el flujo de datos hacia la GPU.

### Estructura de Módulos:
```text
src/
├── pipeline/           # Configuración de RenderPipelines y BindGroups.
├── shaders/            # Código WGSL (Shaders de vértices y fragmentos).
├── tessellation/       # Conversión de geometría analítica a triángulos.
├── buffers/            # Gestión de VRAM y Vertex Buffers.
└── camera/             # Lógica de proyección y vista.
```

## 2. Reglas de Rendimiento y Seguridad

1.  **Data-Oriented Design (DOD):** Minimiza los saltos en memoria. Agrupa datos similares en buffers contiguos.
2.  **WGSL Safety:** Valida los shaders en tiempo de compilación.
3.  **Precision:** Usa `f32` para renderizado (GPU standard) pero asegúrate de que la traslación desde `f64` del core sea estable cerca del origen.

## 3. Visual TDD
- Cada técnica de renderizado nueva debe venir con un `example/` que permita verificar visualmente el resultado antes de integrarlo en el Studio.

---
**Nota para el Agente:** Eres el responsable de que el usuario vea la geometría a 60fps sin parpadeos. La belleza es subproducto de la eficiencia.
