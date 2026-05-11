---
name: fcad-studio-architecture
description: "Trigger: fcad-studio, Preact, Signals, UI state, tools, command palette. Enforce Studio frontend architecture."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-studio-architecture

## Activation Contract

Use this skill for changes in `fcad-studio`, including Preact components, Signals state, CAD tools, command palette, viewport UI, panels, and Tauri API wrappers.

## Hard Rules

- Studio captures intention and renders derived state; it must not own CAD geometry or semantic document truth.
- Use Preact + `@preact/signals`; avoid React-only patterns and avoid `useState`/`useEffect` when Signals fit.
- Apply incoming `DomainEventBatch` updates inside `batch()`.
- Components must not call Tauri `invoke()` directly; use typed module APIs.
- Common UI primitives must come from shadcn/ui + Tailwind v4, not custom rebuilt widgets.
- CAD tools produce `InputIntent`, preview requests, and `DomainCommand`s; they do not persist geometry in TS.

## Decision Gates

| Need | Action |
|------|--------|
| Store panel/tool state | Use Signals as visual/ephemeral state. |
| Store document geometry | Do not store; request Core snapshot/events. |
| Add UI primitive | Import/customize shadcn/ui. |
| Add CAD action | Build command via typed command API. |

## Execution Steps

1. Classify state as UI, tool, view-derived, or forbidden domain truth.
2. Keep module boundaries through public `index.ts` APIs.
3. Emit commands through typed gateway wrappers.
4. Update view state from events/snapshots, not local guesses.

## Output Contract

Return state ownership, command path, module boundary decisions, and tests/verification.

## References

- `docs/architecture/studio-ui-state-model.md`
- `docs/architecture/ipc-command-gateway.md`
- `fcad-studio/AGENTS.md`
