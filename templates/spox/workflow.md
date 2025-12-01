# Spec Oxide Workflow

**Mindset:** Specs are truth. Changes are proposals. Keep in sync.

## The Laws

The user SHALL clarify gaps or ambiguities. Ask questions, NEVER assume.

## Quick Reference

```bash
spox config show              # Get folder paths (authoritative)
spox spec list                # List specs
spox spec show <id>           # View spec
spox change list              # List active changes
spox change show <id>         # View change (--json --deltas-only for parsing)
spox change validate          # Validate before sharing
rg -n "pattern" specs         # Full-text search
```

## Decision Tree

```
New request?
├─ Bug fix / typo / config / dep update → Fix directly
├─ New feature / breaking change / architecture / optimize performance / update security → Create proposal
└─ Unclear → Create proposal (safer)
```

## Before Any Task

**Context Checklist:**
- [ ] Run `spox change list` to see active changes
- [ ] Run `spox spec list` to see existing capabilities
- [ ] Read relevant specs in `specs/[capability]/spec.md`
- [ ] Check pending changes in `changes/` for conflicts
- [ ] Read `specs/mission.md` for conventions

**Before Creating Specs:**
- Always check if capability already exists
- Prefer modifying existing specs over creating duplicates
- Use `spox spec show <id>` to review current state
- If request is ambiguous, ask 1–2 clarifying questions before scaffolding

## Stage 1: Create Proposal
Use command `/spox:propose`

**Before starting:**
1. `spox config show` → get paths
2. `spox spec list` + `spox change list` → check existing
3. Read `specs/mission.md`

**Scaffold** `specs/_changes/<change-id>/`:
- `proposal.md` — why + what
- `tasks.md` — implementation checklist
- `design.md` — only if: cross-cutting, new deps, security/perf, ambiguity
- `specs/<capability>/spec.md` — delta specs

**Change ID:** kebab-case, verb-led (`add-`, `update-`, `remove-`, `refactor-`), unique

**Delta format:**
```markdown
## ADDED|MODIFIED|REMOVED|RENAMED Requirements

### Requirement: Name
Description using SHALL/MUST.

#### Scenario: Description
- **WHEN** condition
- **THEN** outcome
```

**Validate:** `spox change validate` → fix all issues → request approval

**Do not implement until approved.**

## Stage 2: Implement
Use command `/spox:implement`

Complete as TODOs:
1. Read `proposal.md`, `design.md` (if exists), `tasks.md`
2. Implement tasks sequentially (spawn subagents to parallelize)
3. Verify all complete before updating
4. Mark tasks `- [x]`
5. Run verification agent

## Stage 3: Archive
Use command `/spox:archive`

After deployment:
1. Move `specs/_changes/<id>/` → `specs/_archive/YYYY-MM-DD-<id>/`
2. Update `specs/` with final changes
3. `spox spec validate`

## Delta Operations

| Operation | Use When |
|-----------|----------|
| ADDED | New standalone requirement |
| MODIFIED | Changed behavior (paste full requirement) |
| REMOVED | Deprecated |
| RENAMED | Name change only |

**MODIFIED pitfall:** Must include full existing requirement text, not partial. Missing text = lost at archive.

## Scenario Format

```markdown
#### Scenario: Name     ✓ Correct (h4)
- **Scenario: Name**    ✗ Wrong
**Scenario**: Name      ✗ Wrong
### Scenario: Name      ✗ Wrong
```

Every requirement needs ≥1 scenario.

## File Rules

### File Purposes
- `proposal.md` - Why and what
- `tasks.md` - Implementation steps
- `design.md` - Technical decisions
- `spec.md` - Requirements and behavior

### File Templates

**Law**: always use these templates for scaffolding, creating or updating the following files:

- Spec Delta / Change proposal
    - `proposal.md` → @.spox/specs/change/proposal.md
    - `design.md` → @.spox/specs/change/design.md
    - `task.md` → @.spox/specs/change/tasks.md
    - `spec.md` → @.spox/specs/change/spec.md
    - `verification.md` → @.spox/specs/change/verification.md
- Specifications
    - `spec.md` → @.spox/specs/spec.md
- Mission Statement
    - `mission.md` → @.spox/specs/mission.md

### File Paths
- Run `spox config show` to learn about the folder for specs and changes.
- The output of `spox config show` is leading.
- This doc uses the defaults  `specs/` and `specs/_changes/`

## Troubleshooting

| Error | Fix |
|-------|-----|
| "must have at least one delta" | Check `specs/_changes/<id>/specs/` has .md with `## ADDED|MODIFIED|...` |
| "must have at least one scenario" | Use `#### Scenario:` (4 hashes, no bullets) |
| Silent parse failure | `spox change show <id> --json --deltas-only` to debug |

## Multi-Capability Example

```
specs/_changes/add-2fa-notify/
├── proposal.md
├── tasks.md
└── specs/
    ├── auth/spec.md          # ADDED: Two-Factor Auth
    └── notifications/spec.md # ADDED: OTP Email
```

## Best Practices

### Clear References
- Use `file.ts:42` format for code locations
- Reference specs as `specs/auth/spec.md`
- Link related changes and PRs

### Capability Naming
- Use verb-noun: `user-auth`, `payment-capture`
- Single purpose per capability
- 10-minute understandability rule
- Split if description needs "AND"

### Change ID Naming
- Use kebab-case, short and descriptive: `add-two-factor-auth`
- Prefer verb-led prefixes: `add-`, `update-`, `remove-`, `refactor-`
- Ensure uniqueness; if taken, append `-2`, `-3`, etc.