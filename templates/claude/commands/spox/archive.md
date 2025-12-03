---
name: "Spec Oxide: Archive"
description: Archive a deployed Spec Oxide change and merge specs into truth.
category: Spec Oxide
tags: [ spox, archive ]
---

## Goal

**Merge completed changes into the source of truth.**

You are archiving a change that has been implemented and deployed. The spec deltas become permanent specifications; the
change folder moves to the archive. This completes the propose → implement → archive lifecycle.

The user provides change IDs to archive, either directly in the prompt or when asked. Only archive changes that have
been implemented (if the user does not overrule).

## Guardrails

- **Implemented changes only.** Do not archive changes that haven't been implemented.
- **Verify before archiving.** Confirm the change is complete and valid.
- **Apply deltas carefully.** The archived deltas become the new source of truth.
- **Check conventions.** Read `.spox/workflow.md` if you need guidance.

## Steps

### 1. Identify Change to Archive

Get the change ID from the user. Look for:

- Change ID in the prompt or `<ChangeId>` block
- Loose reference (title, summary) → run `spox change list` → confirm with user
- No reference → ask user which deployed change to archive

If you cannot identify a single change ID, stop and ask for clarification.

### 2. Validate the Change

Confirm the change is ready to archive:

```bash
spox change show <id>
spox change validate
```

**Stop if:**

- Change doesn't exist
- Change is already archived
- Validation fails
- Tasks in `tasks.md` aren't all marked `- [x]`

### 3. Archive the Change

Move the change folder to the archive with today's date:

```
specs/_changes/<id>/  →  specs/_archive/YYYY-MM-DD-<id>/
```

### 4. Apply Deltas to Specs

Update the source of truth by applying each delta:

| Delta Operation            | Action                                               |
|----------------------------|------------------------------------------------------|
| `## ADDED Requirements`    | Add new requirements to `specs/<capability>/spec.md` |
| `## MODIFIED Requirements` | Replace existing requirements with updated versions  |
| `## REMOVED Requirements`  | Remove requirements from specs                       |
| `## RENAMED Requirements`  | Update requirement names                             |

**Rule**: Always adhere to the template for specs: @.spox/specs/spec.md - update spec.md files to match the template.

### 5. Verify Final State

Confirm specs are consistent:

```bash
spox spec validate
spox spec list
```

Fix any validation issues before completing.

## Output

When complete:

- Change folder moved to `specs/_archive/YYYY-MM-DD-<id>/`
- Spec deltas applied to `specs/<capability>/spec.md`
- `spox spec validate` passes

The change is now part of the permanent specification. Ready for the next proposal.

## Quick Reference

```bash
spox change list        # Find change IDs
spox change show <id>   # Review before archiving
spox change validate    # Verify change is valid
spox spec validate      # Verify specs after archiving
spox spec list          # Confirm updated capabilities
```