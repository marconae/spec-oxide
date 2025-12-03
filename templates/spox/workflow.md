# Spec Oxide Workflow

## Why Spec-Driven Development?

AI coding assistants are powerful but unpredictable when requirements live only in chat history. Spec Oxide locks intent
before implementation: you agree on *what* to build before writing any code, giving you deterministic, reviewable
outputs.

**Core principle:** Specs are the source of truth. Changes are proposals that modify that truth. Keep them in sync.

**Two-folder model:**

- `specs/mission.md` — Project mission for conventions, goals, tech-stack, high-level architecture
- `specs/` — current truth (what the system does now) defined as capability specs
- `specs/_changes/` — proposed updates (spec deltas) (what you want to change)

This separation keeps diffs explicit and makes multi-spec updates manageable.

**Template for specs:**

Use this template for spec documents for the current truth: `.spox/specs/spec.md`

```markdown

## Commands

Use the `spox` CLI to list, view, and validate specs and changes:

```bash
spox config show              # Paths (authoritative—use these, not defaults)
spox spec list                # Current specifications
spox spec show <id>           # View a spec
spox change list              # Active change proposals  
spox change show <id>         # View a change (add --deltas-only to debug)
spox change validate          # Validate before requesting approval
spox spec validate            # Validate after archiving
```

For full-text search across specs: `rg -n "pattern" specs`

## When to Create a Proposal

```
New request?
├─ Bug fix restoring intended behavior  → Fix directly
├─ Typo / comment / formatting          → Fix directly  
├─ Config change / dep update           → Fix directly
├─ New feature or capability            → Create proposal
├─ Breaking change (API, schema)        → Create proposal
├─ Architecture or pattern change       → Create proposal
└─ Unclear?                             → Create proposal (safer)
```

## Stage 1: Propose

**Goal:** Agree on what to build before writing code.

### Before You Start

1. `spox config show` — get authoritative paths
2. `spox spec list` + `spox change list` — understand current state
3. Read `specs/mission.md` — project conventions and context

### Create the Change

**Directory:** `specs/_changes/<change-id>/`

**Change ID rules:** kebab-case, verb-led, unique

- Good: `add-two-factor-auth`, `update-payment-flow`, `remove-legacy-api`
- Bad: `auth-stuff`, `fix`, `changes`

**Required files:**

| File                         | Purpose                                          |
|------------------------------|--------------------------------------------------|
| `proposal.md`                | Why this change? What's the impact?              |
| `tasks.md`                   | Ordered implementation checklist                 |
| `specs/<capability>/spec.md` | Delta specs (one folder per affected capability) |

**Optional file:**

| File        | When to Include                                                                                  |
|-------------|--------------------------------------------------------------------------------------------------|
| `design.md` | Cross-cutting changes, new dependencies, security/perf concerns, or ambiguity needing resolution |

**File templates:**

Use these templates for scaffolding:
* `.spox/specs/change/proposal.md`
* `.spox/specs/change/tasks.md`
* `.spox/specs/change/design.md` (if needed)
* `.spox/specs/change/spec.md`

### Write Delta Specs

Deltas describe *changes* to existing specs, not the full spec. Place them in
`specs/_changes/<id>/specs/<capability>/spec.md`.

**Format:**

```markdown
## ADDED Requirements

### Requirement: Two-Factor Authentication

Users SHALL provide a second factor during login.

#### Scenario: OTP challenge on valid credentials

- **WHEN** valid username and password provided
- **THEN** system prompts for OTP
- **AND** login completes only after valid OTP
```

**Operations:**

| Header                     | Use When                   |
|----------------------------|----------------------------|
| `## ADDED Requirements`    | New standalone capability  |
| `## MODIFIED Requirements` | Changing existing behavior |
| `## REMOVED Requirements`  | Deprecating functionality  |
| `## RENAMED Requirements`  | Name change only           |

**Critical for MODIFIED:** Paste the *complete* existing requirement, then edit. Partial text = content lost at archive
time.

### Scenario Syntax

Every requirement needs at least one scenario. Format matters:

```markdown
#### Scenario: Name ✓ (h4 header)

- **Scenario: Name**    ✗ (bullet)
  **Scenario**: Name ✗ (bold text)

### Scenario: Name ✗ (h3 header)
```

### Validate and Share

```bash
spox change validate    # Must pass before requesting approval
```

**Stop here.** Do not implement until the proposal is reviewed and approved.

---

## Stage 2: Implement

**Goal:** Build exactly what was approved.

Work through these steps as TODOs:

1. **Read** `proposal.md` → understand the why
2. **Read** `design.md` (if present) → understand technical decisions
3. **Read** `tasks.md` → get the implementation checklist
4. **Implement** tasks in order (parallelize via subagents where independent)
5. **Verify** all tasks complete before updating status
6. **Update** `tasks.md` — mark each item `- [x]`
7. **Run** verification agent for final correctness check

---

## Stage 3: Archive

**Goal:** Merge approved changes into the source of truth.

After deployment:

1. Move `specs/_changes/<id>/` → `specs/_archive/YYYY-MM-DD-<id>/`
2. Apply deltas to `specs/<capability>/spec.md` files
3. Run `spox spec validate` to confirm specs are consistent

---

## Multi-Capability Example

When a change affects multiple capabilities, create one delta file per capability:

```
specs/_changes/add-2fa-notify/
├── proposal.md
├── tasks.md
└── specs/
    ├── auth/
    │   └── spec.md       # ADDED: Two-Factor Authentication
    └── notifications/
        └── spec.md       # ADDED: OTP Email Notification
```

---

## Troubleshooting

| Error                                 | Cause                       | Fix                                                                                        |
|---------------------------------------|-----------------------------|--------------------------------------------------------------------------------------------|
| "must have at least one delta"        | Missing or empty spec files | Ensure `specs/_changes/<id>/specs/` has `.md` files with `## ADDED\|MODIFIED\|...` headers |
| "must have at least one scenario"     | Wrong scenario format       | Use `#### Scenario:` (4 hashes, no bullets, no bold)                                       |
| Silent parse failure                  | Malformed delta             | Run `spox change show <id> --json --deltas-only` to inspect parsed output                  |
| Validation passes but content missing | Partial MODIFIED            | Paste full requirement text before editing                                                 |

---

## Quick Checklist

**Before proposing:**

- [ ] Checked `spox change list` for conflicts
- [ ] Checked `spox spec list` for existing capabilities
- [ ] Read `specs/mission.md`

**Before requesting approval:**

- [ ] `spox change validate` passes
- [ ] Every requirement has ≥1 scenario
- [ ] Change ID is verb-led and unique

**Before marking implementation complete:**

- [ ] All tasks in `tasks.md` marked `- [x]`
- [ ] Verification agent confirms correctness

**Before archiving:**

- [ ] Change is deployed
- [ ] Deltas applied to `specs/`
- [ ] `spox spec validate` passes