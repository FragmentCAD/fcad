# 🤖 Agent Guidelines: fcad-core (The Brain)

Este documento define las reglas de juego estrictas para cualquier Agente de IA que escriba código dentro de este repositorio (`fcad-core`).

**Contexto:** Este es el backend puro en Rust. Maneja matemáticas, memoria (ECS) y comunicación con IA. Debe ser **Headless** (no sabe qué es una ventana ni una GPU).

---

## 1. Filosofía de Código

*   **100% Rust Seguro:** Evita `unsafe` a menos que sea una optimización SIMD crítica y esté justificada con un comentario `// SAFETY: ...`.
*   **Zero-Panic:** El núcleo nunca debe entrar en pánico (`panic!`). Usa `Result<T, E>` para todo. Si un cálculo matemático falla (ej. división por cero), devuelve un error controlado.
*   **No `println!`:** Este proceso se comunica por `stdio` con la IA. Si imprimes algo que no sea JSON, rompes el protocolo. Usa `tracing::info!` o `tracing::error!` (que van a `stderr`).

## 2. Arquitectura: Hexagonal + Domain Screaming

Este repositorio debe seguir un patrón **Hexagonal** para proteger el núcleo geométrico y **Screaming Architecture** para reflejar el dominio CAD.

```text
src/
├── domain/            # El CORAZÓN. Lógica pura de CAD, matemáticas y geometría.
│   ├── architecture/  # Entidades de arquitectura (Walls, Openings).
│   ├── math/          # Primitivas puras (Point2D, Line) y operaciones.
│   └── generators/    # Algoritmos de generación paramétrica.
├── application/       # Orquestación. Casos de uso que coordinan el dominio.
├── infrastructure/    # Adaptadores de SALIDA (Persistencia, RAG, Assets).
└── mcp/               # Adaptador de ENTRADA. Servidor JSON-RPC 2.0 (Tokio).
```

### Reglas de Dependencia:
1.  **Hacia Adentro:** El `domain` no puede importar nada de `application`, `infrastructure` o `mcp`.
2.  **Ports & Adapters:** Usa Traits para definir interfaces de infraestructura (ej. `StandardsProvider`) que el dominio consume pero no implementa.
3.  **ECS como Estado:** El ECS (Bevy) es el mecanismo de persistencia en memoria y gestión de estado. La lógica de dominio debe ser capaz de operar sobre componentes del ECS.

## 3. Reglas de Concurrencia (Tokio vs Bevy)

El servidor MCP corre en un Runtime asíncrono (`tokio`). El ECS corre en un bucle síncrono.

*   **Prohibido:** Mutar el `World` directamente desde una tarea `async`.
*   **Obligatorio:** Usar canales `mpsc` para enviar comandos de intención (`McpCommand`) desde el servidor hacia el bucle principal.

## 4. Flujo de Trabajo (Cargo)

*   Compilar: `cargo build`
*   Testear: `cargo test` (TDD obligatorio para matemáticas)
*   Ejecutar Servidor: `cargo run -- serve`

## 5. Pruebas (TDD)

*   **Matemáticas:** Cada primitiva debe tener tests unitarios en su mismo archivo (`mod tests`).
*   **Integración:** Los tests de integración que requieren levantar un proceso viven en `fcad-agent-skills`, no aquí. Aquí solo testeamos la lógica interna.

---

**Nota Final:** Eres el guardián de la verdad geométrica. Prioriza la precisión (`f64`) y la estabilidad de memoria sobre cualquier otra cosa.
