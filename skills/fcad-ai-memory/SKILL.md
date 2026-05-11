---
name: fcad-ai-memory
description: "Trigger: AI memory, context store, RAG, retrieval, Engram, observations. Enforce FragmentCAD AI memory model."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-ai-memory

## Activation Contract

Use this skill for changes involving AI Context Store, retrieval, RAG, embeddings, Engram adapters, observations, summaries, user/project preferences, or AI decision memory.

## Hard Rules

- AI memory is derived and consultive; it is never CAD document authority.
- Store semantic observations with `scope`, `type`, `topic_key`, source, timestamps, and refs to entities/assets/commands/documents.
- Prefer SQLite + FTS5 first; add vector search only for proven retrieval needs.
- Do not store hover, mouse moves, transient previews, renderer buffers, secrets, or raw high-frequency geometry noise.
- Retrieval returns context and references, never mutations; proposals still pass through Core validation.
- Engram integration must be an optional adapter/import-export path, not a hard runtime dependency.

## Decision Gates

| Need | Action |
|------|--------|
| Remember preference/decision | Save typed observation with stable topic key. |
| Query project context | Retrieve scoped observations + refs. |
| Need embeddings | Prove FTS/structured query is insufficient first. |
| Share with code-agent memory | Use Engram adapter, not direct authority coupling. |

## Execution Steps

1. Classify memory by scope/type and source.
2. Attach refs to Core entities/assets/commands where available.
3. Keep retrieval output auditable and bounded.
4. Ensure resulting agent action goes through proposal/validation.

## Output Contract

Return observation schema impact, storage/retrieval path, excluded data, and validation boundary.

## References

- `docs/architecture/ai-context-store.md`
- `docs/architecture/fcad-ai-package-model.md`
- `docs/architecture/shared-contracts-model.md`
