---
name: fcad-ai-architecture
description: "Trigger: fcad-ai, agent, planning, SOLO, boosts, MCP, AI workflow. Enforce AI package authority boundaries."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

# Skill: fcad-ai-architecture

## Activation Contract

Use this skill for changes involving `fcad-ai`, agent orchestration, planning, SOLO mode, boosts/CLI, MCP AI tools, agent interaction, proposals, policies, or AI audit.

## Hard Rules

- `fcad-ai` is a cognitive client of Core; it never mutates ECS directly.
- Agent outputs must be `IntentPlan`, proposal, preview request, or commit request validated by Core.
- SOLO is an agent execution mode under `agent/modes/solo`, not a separate subsystem or Core bypass.
- MCP and CLI boosts are adapters over internal capabilities; they do not define the runtime.
- Policies govern autonomy and risk; audit records plans, validations, warnings, approvals, and applied commands.

## Decision Gates

| Need | Action |
|------|--------|
| User-agent interaction | Use typed `AgentInteraction`/`AgentResponse`. |
| Generate project/layout | Produce staged semantic proposals before commands. |
| Fast autonomous iteration | Use SOLO mode with policies, validation, rollback, report. |
| External agent access | Expose adapter over same internal workflow. |

## Execution Steps

1. Place logic in agent, planning, core-client, interaction, policies, audit, adapters, or contracts by responsibility.
2. Route all materialization through Core validation.
3. Record audit and memory-worthy decisions.
4. Keep Studio as UX mediator, not agent brain.

## Output Contract

Return package area touched, proposal/validation path, autonomy policy, and audit/memory impact.

## References

- `docs/architecture/fcad-ai-package-model.md`
- `docs/architecture/agent-workflow-and-solo-mode.md`
- `docs/architecture/ai-runtime-bridge.md`
