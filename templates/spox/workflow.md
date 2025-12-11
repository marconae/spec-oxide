## Why Spec-Driven Development?

You are working on a project that uses spec-driven development with Spec Oxide (short SpOx). Spec Oxide locks intent
before implementation: you agree on *what* to build before writing any code, giving you deterministic, reviewable
outputs.

**Core principle:** Specs are the source of truth. Changes are proposals that modify that truth. Keep them in sync.

**Two-folder model:**

- `specs/mission.md` — Project mission for conventions, goals, tech-stack, high-level architecture
- `specs/` — current truth (what the system does now) defined as capability specs
- `specs/_changes/` — proposed updates (spec deltas) (what you want to change)

This separation keeps diffs explicit and makes multi-spec updates manageable.

**MCP tools:** Use Spox MCP tools for all spec and change operations. See `standards/mcp.md` for tool reference.

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

1. Read `specs/mission.md` — project conventions and context
2. Use `mcp__spox__list_specs` and `mcp__spox__list_changes` to check existing state
3. Use `mcp__spox__search_specs` to find relevant specs

### Create the Change

**Directory:** `specs/_changes/<change-id>/`

**Change ID rules:** kebab-case, verb-led, unique

- Good: `add-two-factor-auth`, `update-payment-flow`, `remove-legacy-api`
- Bad: `auth-stuff`, `fix`, `changes`

**Required files:**

- `proposal.md` — Why this change? What's the impact?
- `tasks.md` — Ordered implementation checklist
- `specs/<capability>/spec.md` — Delta specs (one folder per affected capability)

**Optional:** `design.md` — Cross-cutting changes, new dependencies, security/perf concerns

**File templates:**

Use these templates for scaffolding:

* `.spox/templates/change/proposal.md`
* `.spox/templates/change/tasks.md`
* `.spox/templates/change/design.md` (if needed)
* `.spox/templates/change/spec.md`

**Note:** Templates are bundled in the `spox` binary. View them with `spox template show <name>` (coming soon).

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
#### Scenario: Name     ✓ (h4 header)

- **Scenario: Name**    ✗ (bullet)
  **Scenario**: Name    ✗ (bold text)

### Scenario: Name      ✗ (h3 header)
```

### Validate and Share

Validate proposals before requesting approval:

Use `mcp__spox__validate_change` to validate the change proposal structure and content.

**Stop here.** Do not implement until the proposal is reviewed and approved.

---

## Stage 2: Implement

**Goal:** Build exactly what was approved.

Work through these steps:

1. Use `mcp__spox__get_change` to retrieve the change (returns `proposal`, `tasks`, `design`, `deltas`)
2. Review: `proposal` → `design` (if present) → `deltas` → `tasks`
3. **Implement** tasks in order (parallelize via subagents where independent)
4. **Verify** all tasks complete before updating status
5. **Update** `tasks.md` — mark each item `- [x]`
6. **Run** verification agent for final correctness check

## Stage 3: Archive

**Goal:** Merge approved changes into the source of truth.

After deployment:

1. Move `specs/_changes/<id>/` → `specs/_archive/YYYY-MM-DD-<id>/`
2. Apply deltas to `specs/<capability>/spec.md` files
3. Use `mcp__spox__validate_spec` to confirm specs are consistent

**Template for specs:** Use `.spox/templates/spec.md` for spec documents.

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

## Troubleshooting

- **"must have at least one delta"** — Ensure `specs/_changes/<id>/specs/` has `.md` files with `## ADDED|MODIFIED|...` headers
- **"must have at least one scenario"** — Use `#### Scenario:` (4 hashes, not bullets or bold)
- **Silent parse failure** — Use `mcp__spox__get_change` to inspect parsed output
- **Validation passes but content missing** — For MODIFIED, paste full requirement text before editing

## Quick Checklist

**Before proposing:**

- [ ] Read `specs/mission.md`
- [ ] Used `mcp__spox__list_changes` to check for conflicts
- [ ] Used `mcp__spox__list_specs` to check existing capabilities

**Before requesting approval:**

- [ ] `mcp__spox__validate_change` passes
- [ ] Every requirement has at least one scenario
- [ ] Change ID is verb-led and unique

**Before marking implementation complete:**

- [ ] All tasks in `tasks.md` marked `- [x]`
- [ ] Verification agent confirms correctness

**Before archiving:**

- [ ] Change is deployed
- [ ] Deltas applied to `specs/`
- [ ] `mcp__spox__validate_spec` passes