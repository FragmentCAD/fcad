# Modelo de estado y arquitectura UI de Studio

**Decisión:** `fcad-studio` captura intención y presenta estado derivado. No posee verdad geométrica ni semántica del documento CAD; esa autoridad vive en `fcad-core`.

## Capas

```text
Shell / Layout
→ Modules
→ Tool Controllers
→ Command Builders
→ IPC Command Gateway
→ Event Handlers
→ Signals de vista
```

## Tipos de estado permitidos

| Estado | Ejemplos | Regla |
|--------|----------|-------|
| UI State | panel abierto, tema, layout, modal | Puede vivir en Signals de Studio. |
| Tool State | herramienta activa, drag temporal, rubber-band | Efímero; no es documento. |
| View State | selección visual, hover, inspector derivado | Se actualiza desde eventos/snapshots del Core. |
| Domain State | geometría, capas reales, entidades, rooms | Prohibido almacenarlo como verdad en TS. |

## Preact Signals

- Usar `signal` para estado visual mutable.
- Usar `computed` para derivaciones UI.
- Usar `batch` al aplicar `DomainEventBatch` para evitar renders intermedios.
- Usar `effect` / `useSignalEffect` solo para side-effects explícitos y cancelables.
- Evitar `useState` / `useEffect` cuando Signals resuelven el caso con menor acoplamiento.

## Organización recomendada

```text
fcad-studio/src/
├── app-shell/
├── core/
│   ├── ipc/              # wrappers Tauri tipados
│   ├── commands/         # builders de DomainCommand
│   ├── events/           # reducers/handlers de eventos
│   └── signals/          # estado visual global
├── modules/
│   ├── viewport/
│   ├── command-palette/
│   ├── domain-tools/
│   ├── properties/
│   ├── explorer/
│   └── ai-console/
└── ui/                   # shadcn/ui adaptado
```

## Reglas duras

1. Componentes Preact no llaman `invoke()` directo; usan APIs tipadas del módulo o `core/ipc`.
2. Herramientas CAD producen `InputIntent` y `DomainCommand`, no geometría persistente en TS.
3. Previews visuales son efímeros y deben confirmarse por Core antes de commitear.
4. Componentes visuales comunes salen de shadcn/ui; no reinventar botones, diálogos, menús o panels base.
5. Módulos importan APIs públicas (`index.ts`), no internals de otros módulos.

## Flujo de herramienta CAD

```text
Pointer/Keyboard Input
→ Tool Controller
→ optional Core Preview Query
→ Visual Preview
→ Command Builder
→ Command Gateway
```

## Checklist

- [ ] ¿El estado guardado en Studio es visual o efímero?
- [ ] ¿La acción termina en un `DomainCommand`?
- [ ] ¿Los eventos del Core se aplican con `batch()`?
- [ ] ¿La UI puede reconstruirse desde snapshot + eventos?
