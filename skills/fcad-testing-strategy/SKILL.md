---
name: fcad-testing-strategy
description: "Trigger: tests, testing, TDD, validation, contracts, fixtures. Enforce FragmentCAD layer-specific testing strategy."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-testing-strategy

## Activation Contract

Use this skill when adding or changing tests, TDD flows, fixtures, IPC contracts, renderer invalidation tests, asset schema validation, or Core command behavior.

## Hard Rules

- New CAD domain mutation requires Core tests that run without UI.
- Studio tests verify command emission, event handling, and view state; they must not encode domain geometry truth.
- Renderer tests verify derived render state, invalidation mapping, dirty work, and examples; avoid requiring real GPU unless necessary.
- Assets/config tests validate schemas, versions, fixtures, and migration behavior.
- IPC tests cover typed payloads, error envelopes, event batches, and snapshot/resync contracts.

## Decision Gates

| Change | Test where |
|--------|------------|
| Domain command/rule | `fcad-core` Rust tests. |
| UI/tool interaction | `fcad-studio` Bun tests. |
| Render invalidation | `fcad-renderer` contract/example tests. |
| YAML/JSON standard | Asset schema validation. |
| Tauri boundary | IPC contract fixture/test. |

## Execution Steps

1. Identify the owning layer and invariant.
2. Write the lowest-level test that proves the rule.
3. Add boundary/contract tests when data crosses packages.
4. Run only relevant commands first; expand verification before final handoff.

## Output Contract

Return tests added/updated, commands run, and invariants protected.

## References

- `docs/architecture/testing-strategy.md`
- `docs/architecture/command-bus-and-events.md`
- `docs/architecture/ipc-command-gateway.md`
