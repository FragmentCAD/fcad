# Modelo de Autoridad ECS

**Decisión:** en FragmentCAD, el `World ECS` de `fcad-core` es la única fuente de verdad del documento CAD. Studio, Renderer, MCP e IA trabajan alrededor de esa verdad; ninguno la reemplaza.

## Contrato principal

| Concepto | Significado en FragmentCAD | Regla |
|----------|----------------------------|-------|
| Entity | Identidad estable de algo del documento | No contiene lógica; solo identifica. |
| Component | Dato de dominio o estado asociado | Geometría, capa, selección, room, wall, metadata. |
| System | Comportamiento que procesa componentes | Snap, selección, persistencia, sync de índices, validación. |
| Command | Intención de cambiar el documento | Única entrada para mutaciones. |
| Event | Hecho ya ocurrido en el dominio | Sale después de mutar el ECS. |
| Invalidation | Consecuencia renderizable | Le dice al renderer qué debe refrescar. |

## Responsabilidades por módulo

| Módulo | Puede hacer | No puede hacer |
|--------|-------------|----------------|
| `fcad-core` | Validar, mutar ECS, emitir eventos, persistir | Depender de Tauri, ventanas o GPU |
| `fcad-studio` | Capturar input, mostrar UI, emitir intenciones | Guardar verdad geométrica paralela |
| `fcad-renderer` | Dibujar buffers derivados y previews | Decidir reglas CAD o ownership semántico |
| `fcad-assets` | Proveer estándares, capas, estilos, bloques | Hardcodear lógica de runtime |
| MCP | Exponer comandos a agentes externos | Ser el bus interno de alta frecuencia |
| IA nativa | Proponer intents/plans verificables | Escribir geometría cruda sin validación |

## Flujo obligatorio

```text
InputIntent
→ DomainCommand
→ ECS Mutation
→ DomainEvent
→ RenderInvalidation
→ Renderer Update
```

Ejemplo:

```text
CreateLineCommand
→ valida puntos, capa activa y constraints
→ crea Entity + Geometry + Layer
→ emite EntityCreated
→ emite RenderInvalidation::GeometryAdded
→ renderer actualiza buffers GPU
```

## Checklist antes de implementar

- [ ] ¿La verdad del documento queda solo en `fcad-core`?
- [ ] ¿La UI emite una intención en vez de mutar estado de dominio?
- [ ] ¿El renderer recibe consecuencias, no reglas de negocio?
- [ ] ¿Snap/hit-test/índices espaciales derivan del ECS?
- [ ] ¿La operación puede testearse sin abrir la UI?
