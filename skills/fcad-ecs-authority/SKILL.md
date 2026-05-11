---
name: fcad-ecs-authority
description: "Trigger: ECS, Core, renderer sync, Tauri commands, CAD state, AI tools. Enforce FragmentCAD state authority rules."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-ecs-authority

## Activation Contract

Use this skill for any change involving `fcad-core`, ECS state, Tauri commands, renderer synchronization, snap/hit-test, MCP, AI tools, CAD document mutations, or architecture docs.

## Hard Rules

- `fcad-core` / `World ECS` is the only source of truth for CAD document state.
- Never duplicate geometry or semantic document state in `fcad-studio`.
- `fcad-renderer` may cache GPU buffers and dirty ranges, but must not own domain state.
- All document mutations must enter as typed `DomainCommand`s.
- ECS mutations must emit `DomainEvent`s and, when visible, `RenderInvalidation`s.
- Spatial indexes, snap caches, and hit-test structures are derived ECS resources, not independent truth.
- MCP is an external adapter over the internal command bus, not the internal runtime.
- AI tools produce intents/plans; Core validates, previews, and commits only through commands.

## Decision Gates

| Need | Action |
|------|--------|
| Change CAD document state | Add or use a `DomainCommand`. |
| Update what the user sees | Emit `RenderInvalidation`. |
| Add AI capability | Model it as intent/plan + validation + preview + commit. |
| Add snap/hit-test data | Derive it from ECS and keep sync explicit. |
| Expose agent access | Build adapter over command bus; do not make MCP the core path. |

## Execution Steps

1. Identify the owning ECS components and systems in `fcad-core`.
2. Route external input through an adapter into a typed command.
3. Validate in Core before mutating the ECS.
4. Emit domain events and render invalidations after mutation.
5. Keep UI state visual-only and renderer state derived-only.
6. Add tests at the Core level for new domain behavior.

## Output Contract

Return the command/event/invalidation path used, any state ownership decision made, and tests or verification performed.

## References

- `docs/architecture/ecs-authority-model.md`
- `docs/architecture/command-bus-and-events.md`
- `docs/architecture/render-invalidation-contract.md`
- `docs/architecture/ai-runtime-bridge.md`
- `docs/roadmap/ai-first-v0-roadmap.md`
