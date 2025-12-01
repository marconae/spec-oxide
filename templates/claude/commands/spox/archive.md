---
name: "Spec Oxide: Archive"
description: Archive a deployed Spec Oxide change and update specs.
category: Spec Oxide
tags: [spox, archive]
---

# Command: Archive

## Guardrails
- Minimal implementation; add complexity only when required
- Tightly scoped to requested outcome
- Check `.spox/workflow.md` for conventions (`ls .spox` if needed)

## Steps

1. **Identify change ID:**
   - `<ChangeId>` block present in prompt or arguments → use that value (trim whitespace)
   - Loose reference → `spox change list` → confirm with user
   - No reference → ask user; wait for confirmation
   - Cannot identify → stop, inform user

2. **Validate:** `spox change show <id>` + `spox change validate <id>`  — stop if missing/archived/not ready

3. **Archive:** Move `specs/_changes/<id>/` → `specs/_archive/YYYY-MM-DD-<id>/`

4. **Update specs:** Apply deltas to `specs/<capability>/spec.md`

5. **Verify:**
   - `spox spec validate`
   - `spox spec list` to confirm updates

## Commands
```bash
spox change list        # Find change IDs
spox change show <id>   # Validate before archive
spox spec validate      # Verify after archive
```