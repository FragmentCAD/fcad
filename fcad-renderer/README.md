
> **Nota de Arquitectura:** Este repositorio es parte del ecosistema FragmentCAD v0.1.0.
> Para entender la visión global, la interacción entre repositorios y las decisiones de diseño (AI-First, ECS, WGPU), visita el repositorio principal: [**fcad-meta**](https://github.com/FragmentCAD/fcad-meta).

---

# `fcad-renderer`: El Ojo (Motor Gráfico Rust)

**Propósito:** Alojar la librería gráfica pura, de bajo nivel, agnóstica a la UI y responsable del renderizado a 60fps+.
**Filosofía:** "El Ojo no piensa, solo dibuja". Independencia total del sistema de ventanas (Tauri/winit) y de la lógica de negocio (Core).
**Tecnologías:** `wgpu` (WebGPU en Rust), WGSL (Shaders), `lyon` (Teselación CPU).

---

## 1. Independencia Total (UI Agnostic)

Este repositorio es una librería de Rust pura (`lib.rs`) que toma datos abstractos (coordenadas, colores, grosores) y los convierte en píxeles sobre una superficie (`wgpu::Surface`).

*   **¿Por qué aislar el renderizado?** Escribir y mantener código de GPU (`wgpu`) es extremadamente verboso (buffers, bind groups, pipelines). Al sacarlo de `fcad-studio` o `fcad-core`, evitamos contaminar la lógica de negocio o de UI con el "ruido" gráfico.
*   **Portabilidad Extrema:** Si en el futuro FragmentCAD se compila para iOS, Android o para la Web (WASM+WebGL), `fcad-renderer` no cambiará una sola línea, ya que `wgpu` compila nativamente a Metal, Vulkan, DX12 o WebGL2 por nosotros.

## 2. El Híbrido: `wgpu` Puro + `lyon`

En lugar de delegar todo el pipeline a motores opacos (como `vello`), construimos la infraestructura de `wgpu` nosotros mismos para tener **Control Total**, y usamos a `lyon` exclusivamente como "Calculadora Matemática".

1.  **Líneas y Curvas Simples (El 90% del CAD):**
    *   No se usa `lyon`. Se inyectan las líneas puras directamente en los `VertexBuffers` de `wgpu`.
    *   **Grosor en Espacio de Pantalla:** Se escriben **Shaders WGSL** personalizados que toman líneas matemáticas (grosor 0) y las expanden en triángulos a nivel de tarjeta gráfica, garantizando que el grosor de línea se mantenga constante al hacer zoom (un requisito sagrado en CAD profesional).
2.  **Curvas Complejas y Texto Relleno:**
    *   Se usa `lyon` en la CPU (Rust). Se le pasa la curva de Bézier (o la fuente TrueType), `lyon` la triangula matemáticamente, y devuelve un array de triángulos. Esos triángulos simples se inyectan en un buffer genérico de `wgpu` para rellenarlos.

## 3. Optimización de Memoria de Video (VRAM Dirty Flags)

Para mantener 60fps constantes al renderizar millones de entidades, `fcad-renderer` no puede "volver a subir" todo el plano a la tarjeta gráfica en cada fotograma.

*   **Sincronización Diferencial:** La API de `fcad-renderer` implementa banderas de suciedad (Dirty Flags). El motor solo sube a la VRAM los vértices de las entidades que han sido creadas, modificadas o borradas en el último frame (ej. la línea que el usuario está arrastrando en ese instante). Todo el resto del plano estático ya vive en la memoria súper-rápida de la GPU.
*   **Batching Rendering:** Agrupa entidades del mismo color, capa o tipo para minimizar los "Draw Calls" de la tarjeta de video, exprimiendo al máximo el rendimiento del hardware.

## 4. Pruebas Aisladas (Visual TDD)

Dado que es un repositorio independiente, puede contener pequeños ejecutables de prueba (`examples/viewer.rs`) que usen una ventana simple (`winit`) para depurar Shaders o efectos visuales en segundos, sin tener que compilar toda la pesada aplicación Tauri (`fcad-studio`).
