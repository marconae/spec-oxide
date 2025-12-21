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

$ARGUMENTS

## Guardrails

- **Implemented changes only.** Do not archive changes that haven't been implemented.
- **Verify before archiving.** Confirm the change is complete and valid.
- **Apply deltas carefully.** The archived deltas become the new source of truth.

## Steps

### 1. Identify Change to Archive

Get the change ID from the user. Look for:

- Change ID in the prompt or `<ChangeId>` block
- Loose reference (title, summary) → use `mcp__spox__list_changes` → confirm with user
- No reference → ask user which deployed change to archive

If you cannot identify a single change ID, stop and ask for clarification.

### 2. Validate the Change

Confirm the change is ready to archive:

**Use Spox MCP tools:**

- `mcp__spox__get_change` with `change_id` parameter — Review the change proposal
- `mcp__spox__validate_change` with `change_id` parameter — Validate the change

**Stop if:**

- Change doesn't exist
- Change is already archived
- Validation fails
- Tasks in the task list aren't all marked `- [x]`

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

**Rule**: Always adhere to the template for specs in `.spox/templates/spec.md` - update spec.md files to match the template.

### 5. Verify Final State

Confirm specs are consistent:

**Use Spox MCP tools:**

- `mcp__spox__validate_spec` — Validate all specs
- `mcp__spox__list_specs` — Confirm updated capabilities

Fix any validation issues before completing.

### 6. Update Search Index

Rebuild the search index so archived changes are reflected in search results:

**Use Spox MCP tools:**

- `mcp__spox__rebuild_index` — Rebuild the search index from all specs

This ensures the index is up-to-date for searching after the change is archived.

## Output

When complete:

- Change folder moved to `specs/_archive/YYYY-MM-DD-<id>/`
- Spec deltas applied to `specs/<capability>/spec.md`
- `spox spec validate` passes

The change is now part of the permanent specification. Ready for the next proposal.