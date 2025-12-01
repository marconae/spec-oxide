---
name: "Spec Oxide: Implement"
description: Implement an approved Spec Oxide change and keep tasks in sync.
category: Spec Oxide
tags: [spox, implement]
---

# Command: Implement

## Guardrails

- Minimal first; add complexity only when required
- Tightly scoped to requested outcome
- Check `.spox/workflow.md` for conventions (`ls .spox` if needed)

## Steps

Complete sequentially as TODOs:

1. Read `specs/_changes/<id>/proposal.md`, `design.md`, `tasks.md` → confirm scope
2. Spawn `implementer` subagent per task (parallelize if possible and as outlined in `task.md`)
3. Verify all tasks complete before updating status
4. Mark each task `- [x]` in `tasks.md`
5. Spawn `implementation-verifier` for final report

## Commands

```bash
spox change list          # list changes
spox change show <id>     # get proposal context
```

## Role

You orchestrate. Subagents implement. Track progress, enforce guardrails.