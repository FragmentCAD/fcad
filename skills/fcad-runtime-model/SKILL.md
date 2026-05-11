---
name: fcad-runtime-model
description: "Trigger: CadEngine, AppState, runtime, locks, snapshots, command bus. Enforce FragmentCAD runtime coordination rules."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-runtime-model

## Activation Contract

Use this skill for changes involving `CadEngine`, Tauri `AppState`, ECS runtime access, locks, snapshots, command dispatch, event queues, document sessions, or runtime coordination.

## Hard Rules

- `AppState` is an adapter host; it must not own CAD document truth.
- Route mutations through `CadEngine.dispatch(command)` or the approved command bus path.
- Keep `World` access short; never hold locks across render frames, slow IO, IPC streaming, or LLM calls.
- Studio, Renderer, AI, MCP, and CLI consume snapshots/events instead of long-lived `World` locks.
- Command execution must update ECS, domain events, render invalidations, and derived indexes as one logical transaction.

## Decision Gates

| Need | Action |
|------|--------|
| Mutate document | Use `CadEngine`/`CommandBus`. |
| Read for external consumer | Return snapshot/DTO, not raw ECS guard. |
| Coordinate renderer/index | Emit ordered events/invalidations. |
| Long operation | Release engine lock before IO/render/LLM. |

## Execution Steps

1. Identify whether the change is command, query, snapshot, queue, or adapter state.
2. Keep authoritative state inside `CadEngine`/Core.
3. Validate lock boundaries and transaction order.
4. Add tests or fixtures for ordering/snapshot expectations.

## Output Contract

Return state ownership, lock/snapshot path, event/invalidation ordering, and verification performed.

## References

- `docs/architecture/cad-engine-runtime-model.md`
- `docs/architecture/command-bus-and-events.md`
- `docs/architecture/ipc-command-gateway.md`
