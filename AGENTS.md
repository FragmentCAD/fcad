# 🤖 Agent Guidelines: Workspace Completo (fcad)

Bienvenido a FragmentCAD, un CAD arquitectónico 100% nativo impulsado por Rust y TS. Eres una Inteligencia Artificial operando dentro de este monorepo.

Este documento establece las **REGLAS GENERALES Y TÉCNICAS FUNDAMENTALES** del ecosistema. Todos los agentes (incluidos sdd-apply, sdd-verify, etc.) **DEBEN** obedecerlas. Existen guías adicionales por subdirectorio (ej. `fcad-core/AGENTS.md`) pero estas mandan sobre todo.

---

## 1. Stack Tecnológico de Implementación ⚠️ (Reglas Duras)

### Frontend (`fcad-studio` y `fcad-studio/src-tauri`):
- **Runtime Web:** `Bun` está estrictamente encriptado en el diseño. **NUNCA utilices `npm`, `yarn` o `pnpm` o la compilación fallará**. Este error ha costado tiempo y CPU de forma continua.
- **Instalar paquetes frontend:** Usa siempre `bun install` o `bun add <paquete>`.
- **Framework UI:** **Preact**. NO USES React puro (`useState`, `useEffect`). Todo el estado debe usar Signals (`@preact/signals`).
- **Librería de Componentes:** Usamos estrictamente **`shadcn/ui`** (construido sobre `@radix-ui`). **ESTÁ ESTRICTAMENTE PROHIBIDO CONSTRUIR COMPONENTES UI DESDE CERO** (botones, modales, menús, etc.) si ya existe un equivalente en la librería. Tu deber es importar el componente base de shadcn y personalizarlo usando *Tailwind*.
- **Tailwind v4:** Utiliza estilos utilitarios directos sobre los componentes.

### Backend (`fcad-core`, `fcad-renderer`):
- **100% Rust Seguro.** (`unsafe` prohibido).
- **Cargo Workspace:** Este es un *monorepo Cargo*. A la hora de agregar dependencias que se repiten en el backend, revisa y añádelas usando `[workspace.dependencies]` en el archivo `/Cargo.toml` raíz, para invocar luego `{ workspace = true }` en los sub-paquetes.
- No añadas "lockfiles" huérfanos.

## 2. Paradigma Arquitectónico (Screaming / Hexagonal)
El proyecto ha sido rediseñado para gritar su propósito:

1.  **Diferenciación Conceptual**: La UI (Studio) no realiza cálculos matemáticos, solo envía comandos IPC nativos. El Motor (Core) no accede a las ventanas; su estado de la memoria es alterado a través del Component System local en el ECS (Bevy).
2.  **No re-escribir lógica de dominio:** Siempre que interactúes con entidades como "Muros" o "Geometrías", hazlo a través de las primitivas de `fcad-core` (geo-types), nunca construyas representaciones paralelas en TS ni escribas wrappers ad-hoc redundantes.

## 3. Dinámica del Orquestador y Reglas Operativas
1.  **TDD Iterativo y Verificado:** Para flujos complejos de diseño (ej. *precision-drawing-engine* e *interactive tests*) usamos metodologías TDD. Si escribes funciones nuevas, implementa la batería de pruebas en Rust.
2.  **Archivos de Datos Inteligentes:** `fcad-assets/` no es solo "recursos". Es la base de datos de contexto arquitectónico y estándares de la empresa para ti. Extrae YAML y JSON de esa carpeta, nunca intentes hardcodear capas arquitectónicas o parámetros de diseño como grosores en el código Rust o TS.

## 4. Troubleshooting Clásico
- *Ooh, me equivoqué de comando:* Si accidentalmente lanzas un subproceso de `npm` o instalador bloqueador de node_modules en la UI, termínalo inmediatamente usando el ID de la interfaz en tu shell asíncrona, limpia la cache (`rm -rf node_modules`) y usa **bun**.
- *Bloqueo de Cargo:* Si `cargo fetch` en Windows da OsError 32 (archivo en uso por el antivirus/indexer local), cancela, espera un par de segundos y vuelve a lanzar el fetch global en el Workspace Root.

Recuerda: **Eres el mantenedor y orquestador técnico de esta solución. Mantén limpio el Workspace y delega de manera quirúrgica.**

## 5. Master Plan y Roadmap (North Star)
La visión arquitectónica completa y sus fases (v0.1.0 -> v1.0) están persistidas en Engram bajo el título **"FragmentCAD Master Plan"** (Topic Key: `architecture/fragmentcad-master-plan`). 
**Siempre** consulta esta memoria o solicita al Orquestador que la recupere de Engram antes de proponer cambios arquitectónicos importantes. Las directrices principales son:
- **FASE 1:** Core, Navegación (Osnaps, Hardware-acceleration) e Interfaz Híbrida (LUI + Paleta Flotante + Layout Anclable `flexlayout-react`).
- **FASE 2:** Infraestructura Semántica (LSP Arquitectónico, ECS con Intención), Interceptor de Comandos y Linter en Tiempo Real (Feedback Visual WGPU).
- **FASE 3:** Geometría Avanzada (Booleanas 2D) y Ecosistema de Agentes Nativos (Lead, Drafting, Compliance) basados en memoria.
Cualquier ciclo de desarrollo (SDD) a corto plazo debe estar enlazado lógicamente a los objetivos descritos en este Master Plan.
