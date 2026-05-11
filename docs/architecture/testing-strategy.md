# Estrategia de testing por capas

**Decisión:** los tests deben proteger invariantes arquitectónicas, no solo funciones. Cada capa valida su responsabilidad y evita duplicar autoridad del Core.

## Matriz

| Capa | Tests principales | Debe probar |
|------|-------------------|-------------|
| `fcad-core` | unit/integration/property tests Rust | comandos, eventos, invariantes ECS, geometría |
| `fcad-renderer` | tests de invalidación, golden buffers, examples visuales | reconstrucción derivada, dirty ranges, pipelines |
| `fcad-studio` | Bun tests, tests de herramientas, signal reducers | emisión de comandos, aplicación de eventos, no dominio duplicado |
| `fcad-assets` | schema validation, fixtures | estándares válidos y migrables |
| IPC/Tauri | contract tests | payloads tipados, errores, snapshot/resync |

## Reglas duras

1. Toda nueva mutación CAD debe tener test de Core sin abrir UI.
2. Todo nuevo evento/invalidation debe tener test de contrato o fixture.
3. Renderer no necesita GPU real para validar mapping de invalidaciones a trabajo pendiente.
4. Studio debe testear que emite comandos correctos, no que “calcula geometría”.
5. Assets configurables deben validarse contra schema antes de ser consumidos.

## Checklist

- [ ] ¿La regla de dominio está protegida en Core?
- [ ] ¿El contrato IPC tiene fixture o test?
- [ ] ¿Renderer prueba estado derivado y no dominio?
- [ ] ¿Studio prueba intención/comando y no verdad CAD?
