
> **Nota de Arquitectura:** Este repositorio es parte del ecosistema FragmentCAD v0.1.0.
> Para entender la visión global, la interacción entre repositorios y las decisiones de diseño (AI-First, ECS, WGPU), visita el repositorio principal: [**fcad-meta**](https://github.com/FragmentCAD/fcad-meta).

---

# `fcad-core`: El Cerebro (Backend en Rust Puro)

**Propósito:** Alojar la lógica de dominio del CAD, la gestión de memoria activa a 60fps, la persistencia, las matemáticas geométricas y el servidor MCP. Es el verdadero "motor" de FragmentCAD.
**Filosofía:** Screaming Architecture, concurrencia híbrida, separación estricta entre memoria viva y persistencia, y 100% Rust Seguro.

---

## 1. Arquitectura de Memoria (ECS + Índice Espacial)

Para soportar manipulación de geometría a 60fps sin bloqueos, `fcad-core` mantiene la verdad del proyecto enteramente en RAM.

*   **Entity Component System (ECS):** Utilizamos librerías como `bevy_ecs`. Las entidades (líneas, arcos, cotas) que el usuario está visualizando o modificando viven en la RAM estructuradas en componentes. Esto maximiza la localidad de caché (Cache Locality) en el procesador.
*   **Árbol R (`rstar`):** Todas las entidades activas se indexan espacialmente en memoria. Esto permite búsquedas instantáneas (ej. "dame todos los vértices cerca de este clic del ratón" para el *Snap*).

## 2. Persistencia Binaria Rápida (Sin Base de Datos Local)

No se utiliza una base de datos local para el estado "en vivo" del proyecto, evitando así cuellos de botella de I/O y problemas de sincronización de estado.

*   **Guardado y Carga (`.fcad`):** Se utiliza serialización binaria ultra-rápida (ej. `bincode`) para volcar el estado completo del ECS a disco en milisegundos.
*   **Deshacer/Rehacer (Undo/Redo):** Al tener todo el estado en memoria de forma determinista, es trivial mantener un historial de instantáneas (snapshots) o "deltas" de memoria.

## 3. Independencia Matemática

El motor es 100% nativo. Todo cálculo, intersección, operación booleana o parsing de archivos se realiza con cajas maduras del ecosistema de Rust.

*   **Parsing DXF:** `dxf-rs` (o similar) lee y escribe la representación externa de los planos.
*   **Matemáticas y Geometría:** `geo` maneja polígonos, intersecciones y áreas complejas directamente.

## 4. Concurrencia Híbrida (`tokio` + `rayon`)

El motor separa sus cargas de trabajo para que la UI y los agentes de IA nunca se congelen mutuamente:
*   **`tokio` (Asíncrono / I/O):** Maneja el servidor MCP (comunicación con IA local), lecturas de archivos pesados y configuración.
*   **`rayon` (Paralelismo de CPU):** Paraleliza cálculos matemáticos pesados (ej. buscar intersecciones entre 10,000 líneas o despachar vértices al renderer) usando todos los núcleos del procesador de forma segura.

## 5. Compilación Dual (CLI vs Librería)

*   **Como CLI (`src/main.rs`):** Permite ejecutar `fcad-core` de forma *headless* (sin interfaz gráfica). Ideal para pruebas TDD extremadamente rápidas, scripts de automatización, y agentes IA puros.
*   **Como Librería (`src/lib.rs`):** Permite ser enlazado e incrustado dentro de `fcad-studio` (Tauri) para la aplicación de escritorio final, compartiendo el acceso a memoria con `fcad-renderer`.
