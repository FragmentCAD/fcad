---
name: fcad-config-assets
description: "Trigger: fcad-assets, config, standards, layers, materials, schemas, user profiles. Enforce configurable standards model."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-config-assets

## Activation Contract

Use this skill for changes involving `fcad-assets`, configurable standards, layers, materials, styles, schemas, user/workspace/project profiles, or future `fcad-config` logic.

## Hard Rules

- Do not hardcode configurable standards such as layers, widths, styles, materials, or templates in TS/Rust.
- `fcad-assets` owns default declarative content; future `fcad-config` owns loading, validation, merge, migration.
- User/workspace/project overrides must be possible without forking defaults.
- YAML/JSON standards need schema, versioning, and actionable validation errors.
- Core consumes validated effective config; Studio edits/presents config but does not decide domain rules.

## Decision Gates

| Need | Action |
|------|--------|
| Add default standard | Put declarative data in `fcad-assets`. |
| Add merge/validation logic | Place in future/config module, not asset data. |
| Add user customization | Respect Default → User → Workspace → Project precedence. |
| Invalid config | Reject before Core consumption. |

## Execution Steps

1. Classify the change as content, schema, validation, merge, or UI editing.
2. Keep defaults immutable unless deliberately versioned/migrated.
3. Validate data before runtime use.
4. Preserve user override paths and deterministic merge behavior.

## Output Contract

Return config layer affected, schema/versioning impact, override behavior, and validation performed.

## References

- `docs/architecture/configuration-and-standards-model.md`
- `fcad-assets/AGENTS.md`
