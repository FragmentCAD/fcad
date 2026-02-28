# FragmentCAD (fcad) - Monorepo

Bienvenido al repositorio unificado de FragmentCAD, el **IDE Arquitectónico Nativamente Inteligente**.

Este monorepo agrupa todos los componentes clave del ecosistema FragmentCAD en un único Cargo Workspace (Rust) y cliente frontend.

## Estructura del Monorepo

*   [`fcad-core/`](./fcad-core/) - **El Cerebro.** Lógica pura de CAD, ECS (Bevy), matemáticas geométricas y servidor JSON-RPC (MCP) para la comunicación con los agentes de IA.
*   [`fcad-renderer/`](./fcad-renderer/) - **El Ojo.** Motor gráfico de alto rendimiento basado en WGPU para renderizar la geometría con precisión paramétrica a 60fps.
*   [`fcad-studio/`](./fcad-studio/) - **El Cuerpo.** Backend de Tauri y Frontend en Preact/TailwindCSS. Es la Interfaz de Usuario (IDE) que envuelve el motor de renderizado y permite las interacciones del humano. 
*   [`fcad-assets/`](./fcad-assets/) - **La Memoria (Datos).** Biblioteca central de configuración, estándares arquitectónicos (JSON/YAML), diccionarios y geometría (DXF).

*(Nota: RAG y Skills de IA se gestionan de forma independiente en el repositorio `fcad-agent-skills`)*

## Stack Tecnológico

1.  **Backend:** Rust (Cargo Workspace)
    *   Arquitectura ECS (`bevy_ecs`) para gestión de memoria.
    *   WGPU para el pipeline de renderizado gráfico nativo.
    *   Tauri v2 (`fcad-studio/src-tauri`) como "pegamento" IPC nativo.
2.  **Frontend:** TypeScript + Bun
    *   Preact + Signals para la reactividad.
    *   TailwindCSS + shadcn/ui para el sistema de diseño.
    *   Vite como bundler.

## Flujo de Trabajo y Ejecución

Debido a que este repositorio contiene tanto código compilado nativo (Rust) como código web empaquetado (TS/Vite), sigue estas instrucciones:

### Dependencias y Compilación

1. **Gestor de Paquetes Frontend:** NO USAR NPM NI YARN. Todo el entorno web en `fcad-studio` usa **Bun**.
   ```bash
   cd fcad-studio
   bun install
   ```

2. **Compilador Backend:** Todas las dependencias de Rust se resuelven a nivel de raíz del workspace.
   ```bash
   cargo fetch
   cargo check
   cargo test --workspace
   ```

### Ejecutar la Aplicación

Para levantar el entorno completo de desarrollo (Frontend UI + Backend Rust/WGPU):

```bash
cd fcad-studio
bun run tauri dev
```

Este comando inicia el servidor Vite en caliente para UI, compila el ejecutable nativo en Rust mediante Tauri y lanza la aplicación de escritorio.

## Lineamientos para Agentes de IA

Consulta los archivos `AGENTS.md` presentes en el contexto de cada subproyecto para reglas específicas:
*   [Guía Core (Matemáticas y ECS)](./fcad-core/AGENTS.md)
*   [Guía Renderer (WGPU)](./fcad-renderer/AGENTS.md)
*   [Guía Studio (Preact y UI)](./fcad-studio/AGENTS.md)

**Regla de Oro General:** Mantén la "Screaming Architecture" (Arquitectura que Grita su dominio) respetando la separación estricta: UI (Studio) no sabe de Matemáticas (Core), Matemáticas no sabe de Píxeles (Renderer).
