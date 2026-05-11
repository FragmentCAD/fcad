---
name: fcad-ipc-boundary
description: "Trigger: Tauri IPC, commands, events, channels, sync. Enforce FragmentCAD command gateway and event stream rules."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-ipc-boundary

## Activation Contract

Use this skill for changes involving Tauri commands, IPC wrappers, frontend/backend sync, event streams, snapshots, document command dispatch, or renderer invalidation transport.

## Hard Rules

- Tauri is an adapter, not domain ownership.
- All CAD document mutations must enter through a typed Command Gateway and reach `fcad-core` as `DomainCommand`s.
- Preact components must not call `invoke()` directly; use typed module/core IPC APIs.
- Prefer Tauri v2 `Channel<T>` for ordered streams: domain events, invalidations, progress, AI output, diagnostics.
- Use snapshots for open/resync/recovery; never repair authoritative document state in Studio.
- IPC payloads need stable types, versioning, and domain/technical error separation.

## Decision Gates

| Need | Action |
|------|--------|
| Mutate document | Dispatch a typed `DomainCommand`. |
| Stream updates | Use typed event/invalidation batches over a channel. |
| Recover from desync | Request snapshot; do not patch UI state manually. |
| Access OS capability | Use Tauri/plugin as adapter only. |

## Execution Steps

1. Identify whether the change is command, event stream, snapshot, or OS adapter.
2. Route mutations through the Command Gateway.
3. Keep TS/Rust payload contracts explicit and testable.
4. Apply incoming event batches atomically in Studio.
5. Forward render consequences as `RenderInvalidation`s.

## Output Contract

Return the command/event/snapshot path used, payload types touched, and sync/error handling decisions.

## References

- `docs/architecture/ipc-command-gateway.md`
- `docs/architecture/command-bus-and-events.md`
- `docs/architecture/render-invalidation-contract.md`
