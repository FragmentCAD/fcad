---
name: fcad-spatial-index
description: "Trigger: SpatialIndex, snap, hit-test, selection, erase, osnap. Enforce derived spatial query architecture."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-spatial-index

## Activation Contract

Use this skill for changes involving `SpatialIndex`, snap/osnap, hit-test, selection windows, erase targets, geometry query acceleration, or tool picking.

## Hard Rules

- `SpatialIndex` is derived from ECS geometry/layer/visibility state; it is never document truth.
- Snap and hit-test return entity references/candidates, not authoritative duplicated geometry.
- Geometry, layer, visibility, and deletion changes must sync or invalidate the spatial index.
- Renderer, snap, and hit-test must operate on compatible document/index/render versions.
- If the index is stale, rebuild/resync; do not patch or guess in Studio.

## Decision Gates

| Need | Action |
|------|--------|
| Pick/select/erase entity | Query derived index, then issue command by entity ref. |
| Geometry changed | Sync/rebuild index from ECS consequence. |
| Divergence detected | Request resync/snapshot before continuing. |
| Tool preview needs snap | Use query service; do not compute domain in TS. |

## Execution Steps

1. Identify ECS components that feed the index.
2. Define invalidation/sync trigger for every relevant mutation.
3. Return stable entity refs and ranked candidates.
4. Verify renderer and query versions cannot silently diverge.

## Output Contract

Return index source, sync trigger, query result contract, and stale-version handling.

## References

- `docs/architecture/spatial-index-and-snap-model.md`
- `docs/architecture/ecs-authority-model.md`
- `docs/architecture/render-invalidation-contract.md`
