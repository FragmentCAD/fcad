# Modelo de contratos compartidos

**Decisión:** los límites entre Studio, Core, Renderer y AI deben depender de contratos explícitos, versionados y testeables. Los tipos compartidos son parte de la arquitectura, no detalles accidentales de IPC.

## Contratos candidatos

| Contrato | Consumidores | Propósito |
|----------|--------------|-----------|
| `DomainCommand` | Studio, AI, Core, MCP/CLI | Unidad de intención mutable |
| `DomainEvent` | Core, Studio, AI memory | Hecho aplicado |
| `RenderInvalidation` | Core, Renderer, Studio bridge | Consecuencia visual |
| `SnapshotDTO` | Studio, Renderer, AI | Vista consistente del documento |
| `ErrorEnvelope` | Todos los adapters | Errores de dominio/técnicos tipados |
| `AgentInteraction` | Studio, AI | Entrada conversacional tipada |
| `AgentResponse` | AI, Studio | Respuesta/propuesta/pregunta/warning |
| `IntentPlan` | AI, Core | Plan materializable validable |

## Ubicación

No crear `fcad-contracts` hasta que haya duplicación real entre paquetes. Mientras tanto, mantener contratos cerca del owner y exponer DTOs estables. Cuando Studio, AI y Core compartan varios tipos, promoverlos a paquete/crate compartido.

## Reglas duras

1. No cruzar boundaries con strings libres si existe un tipo de dominio.
2. Todo contrato externo debe tener versión o estrategia de compatibilidad.
3. No filtrar tipos internos del ECS si el consumidor necesita un DTO estable.
4. Los errores deben distinguir dominio, validación, autorización, sincronización y fallos técnicos.
5. Los contratos críticos deben tener fixtures o tests de serialización.

## Checklist

- [ ] ¿El boundary usa tipos explícitos?
- [ ] ¿El consumidor necesita DTO y no tipo interno?
- [ ] ¿El contrato tiene tests/fixtures?
- [ ] ¿La evolución del tipo no rompe adapters silenciosamente?
