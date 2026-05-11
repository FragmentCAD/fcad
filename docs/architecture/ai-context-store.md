# AI Context Store y memoria del agente

**Decisión:** la memoria IA de FragmentCAD debe ser persistente, consultiva y derivada. Se inspira en Engram, pero se adapta al dominio CAD con referencias a documentos, entidades, comandos, assets y decisiones de proyecto.

## Principio

```text
DomainEvents + Snapshots + Assets + User Decisions
→ AI Context Store
→ Retrieval
→ Agent Proposal
→ Core Validation
```

El store puede estar incompleto o atrasado; el Core no. Por eso nunca es autoridad de dominio.

## Modelo de observación

```text
AIObservation
├── id
├── scope: user | workspace | project | document
├── type: decision | constraint | preference | issue | proposal | standard | summary
├── topic_key
├── refs: EntityId[] | CommandId[] | AssetId[] | DocumentId[]
├── content
├── source
└── timestamps
```

## Qué guardar

| Tipo | Ejemplos |
|------|----------|
| Semántica del documento | rooms, walls, openings, zones, relaciones espaciales |
| Decisiones | propuesta aceptada/rechazada y motivo |
| Preferencias | estilo del usuario/estudio/proyecto |
| Estándares efectivos | capas, materiales, mínimos, perfiles desde assets/config |
| Summaries | resúmenes de sesión, etapa o alternativa |

## Qué no guardar

- Mouse moves, hover, selección transitoria o previews efímeros.
- Buffers renderer o estado visual puro.
- Geometría bruta de alta frecuencia sin semántica.
- Secretos, paths sensibles o logs gigantes sin estructura.

## Estrategia técnica

1. Empezar con SQLite + FTS5 + metadatos estructurados.
2. Agregar `sqlite-vec`/embeddings cuando existan preguntas reales que FTS no resuelva.
3. Mantener adapters opcionales para importar/exportar con Engram.
4. Usar `topic_key` para upserts de preferencias/decisiones evolutivas.

## Checklist

- [ ] ¿La memoria guarda semántica, no ruido?
- [ ] ¿Cada observación tiene scope, type y refs?
- [ ] ¿Retrieval devuelve contexto y referencias, no mutaciones?
- [ ] ¿La propuesta resultante pasa por Core?
