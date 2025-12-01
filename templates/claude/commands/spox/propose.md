---
name: "Spec Oxide: Propose"
description: Scaffold a new Spec Oxide change and validate strictly.
category: Spec Oxide
tags: [spox, propose]
---

# Command: Propose

## Guardrails
- Minimal first; add complexity only when required
- Tightly scoped to requested outcome
- Ask clarifying questions before editing files
- **No code** — only docs (proposal.md, tasks.md, design.md, spec deltas)
- Check `.spox/workflow.md` for conventions (`ls .spox` if needed)
- Use file templates

## Steps

1. **Context:** Read `specs/mission.md`, run `spox change list`, `spox spec list`, search with `rg`/`ls`

2. **Scaffold** `specs/_changes/<change-id>/`:
    - Choose unique verb-led ID (`add-`, `update-`, `remove-`, `refactor-`)
    - Create `proposal.md`, `tasks.md`
    - Create `design.md` only if: multi-system, new patterns, or trade-offs needed

3. **Spec deltas** in `specs/_changes/<id>/specs/<capability>/spec.md`:
    - One folder per capability
    - Use `## ADDED|MODIFIED|REMOVED Requirements`
    - ≥1 `#### Scenario:` per requirement
    - Cross-reference related capabilities

4. **Tasks** in `tasks.md`:
    - Ordered, small, verifiable items
    - Include validation (tests/tooling)
    - Mark dependencies and parallelizable work

5. **Validate:** `spox change validate` → fix all issues before sharing

## Commands

```bash
spox change list                          # Active changes
spox spec list                            # Existing specs
spox change validate                      # Validate proposal
spox change show <id>                     # Debug failures
rg -n "Requirement:|Scenario:" specs      # Search existing
```


