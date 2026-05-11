---
name: fcad-renderer-architecture
description: "Trigger: fcad-renderer, WGPU, renderer, buffers, shaders, invalidation. Enforce derived renderer architecture."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-renderer-architecture

## Activation Contract

Use this skill for changes in `fcad-renderer`, render synchronization, WGPU pipelines, WGSL shaders, tessellation, GPU buffers, camera, previews, and invalidation handling.

## Hard Rules

- Renderer state is derived and rebuildable; it is not CAD document truth.
- Consume `RenderInvalidation`s; do not poll or infer semantic changes.
- Separate persistent document rendering from preview, hover, selection, and debug layers.
- GPU buffers, dirty ranges, and batches may be cached but must be reconstructable from snapshot/events.
- Renderer must not decide layers, snap, constraints, compliance, or entity ownership.
- Avoid long per-frame locks; prefer compact snapshots and invalidation queues.

## Decision Gates

| Need | Action |
|------|--------|
| Visual update after domain change | Add/use explicit `RenderInvalidation`. |
| Performance cache | Make it derived and rebuildable. |
| Preview/hover/selection | Put in ephemeral layer. |
| Domain rule needed | Move decision to `fcad-core`. |

## Execution Steps

1. Identify the invalidation source and render layer.
2. Map invalidation to dirty work and GPU resource updates.
3. Keep tessellation/render data separate from domain authority.
4. Add diagnostics or examples for new rendering techniques.

## Output Contract

Return invalidations consumed, render layers touched, GPU lifecycle decisions, and visual/test verification.

## References

- `docs/architecture/renderer-derived-state-model.md`
- `docs/architecture/gpu-resource-lifecycle.md`
- `docs/architecture/render-invalidation-contract.md`
- `fcad-renderer/AGENTS.md`
