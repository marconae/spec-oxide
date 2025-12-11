---
name: "Spec Oxide: Propose"
description: Create a change proposal that locks intent before implementation.
category: Spec Oxide
tags: [ spox, change, proposal ]
---

## Goal

**Agree on what to build before writing any code.**

You are creating a proposal—a contract between human and AI that defines exactly what will change. No implementation
happens until this proposal is approved. This prevents wasted effort and ensures alignment.

The user provides input via conversation prompts or by sharing a Markdown file containing a rough idea. Your job is to
transform that input into a structured, validated proposal.

$ARGUMENTS

## Guardrails

- **No code.** Only create documentation: `proposal.md`, `tasks.md`, `design.md`, and spec deltas.
- **Minimal scope.** Keep the proposal tightly focused on the requested outcome.
- **Clarify first.** If anything is vague or ambiguous, ask questions before creating files.

## Steps

### 1. Understand Current State

Before proposing anything, know what exists:

**Use Spox MCP tools:**

- `mcp__spox__list_specs` — List all specs with ID, title, purpose
- `mcp__spox__list_changes` — Check for in-progress changes that might conflict
- `mcp__spox__search_specs` — Find relevant specs semantically
- `mcp__spox__get_spec_requirements` — Explore specific spec requirements

**Important** Use `mcp__spox__search_specs` to search before loading all specs into the context. The search helps to
find relevant specs and avoid loading unnecessary data.

### 2. Choose a Change ID

Format: **verb-led, kebab-case, unique**

| ✓ Good                | ✗ Bad          |
|-----------------------|----------------|
| `add-two-factor-auth` | `auth-changes` |
| `update-payment-flow` | `fix`          |
| `remove-legacy-api`   | `stuff`        |

### 3. Scaffold the Change

Create `specs/_changes/<change-id>/` with:

| File                         | Purpose                                   | Required        |
|------------------------------|-------------------------------------------|-----------------|
| `proposal.md`                | Why this change? What's the impact?       | Yes             |
| `tasks.md`                   | Ordered, verifiable implementation steps  | Yes             |
| `specs/<capability>/spec.md` | Delta specs (one per affected capability) | Yes             |
| `design.md`                  | Technical decisions, trade-offs           | Only if complex |

**When to include `design.md`:** Multi-system changes, new dependencies, security/performance concerns, or ambiguity
that needs resolution before coding.

**Always use these file templates for scaffolding:**

* `.spox/templates/change/proposal.md`
* `.spox/templates/change/tasks.md`
* `.spox/templates/change/design.md` (if needed)
* `.spox/templates/change/spec.md`

### 4. Identify All Affected Capabilities

**Before writing any deltas**, analyze the change to identify ALL capabilities that will be affected.

Use `mcp__spox__list_specs` and `mcp__spox__search_specs` to find capabilities related to the change. Ask yourself:

- Which existing specs need MODIFIED or REMOVED requirements?
- Does this change introduce a NEW capability (new spec)?
- Does this change touch multiple systems (auth, notifications, API, etc.)?

**Create one delta file per affected capability:** `specs/_changes/<change-id>/specs/<capability>/spec.md`

Example for a change affecting two capabilities:

```
specs/_changes/add-2fa-notify/
└── specs/
    ├── auth/
    │   └── spec.md       # ADDED: Two-Factor Authentication
    └── notifications/
        └── spec.md       # ADDED: OTP Email Notification
```

### 5. Write Spec Deltas

For each affected capability, create a delta in `specs/_changes/<change-id>/specs/<capability>/spec.md`.

```markdown
## ADDED Requirements

### Requirement: Two-Factor Authentication

Users SHALL provide a second factor during login.

#### Scenario: OTP required after password

- **WHEN** valid credentials provided
- **THEN** system prompts for OTP
- **AND** login completes only after valid OTP
```

**Rules:**

- Use `## ADDED | MODIFIED | REMOVED | RENAMED Requirements` headers
- Every requirement needs at least one `#### Scenario:` (4 hashes, not bullets)
- Use SHALL/MUST for normative requirements
- For MODIFIED: paste the *complete* existing requirement, then edit
- **One delta file per capability**—never combine capabilities in a single spec.md

### 6. Write Tasks

Create `tasks.md` with ordered, verifiable work items:

```markdown
## 1. Core Implementation

- [ ] 1.1 Add OTP generation service
- [ ] 1.2 Create OTP verification endpoint
- [ ] 1.3 Update login flow to require OTP

## 2. Testing

- [ ] 2.1 Unit tests for OTP service
- [ ] 2.2 Integration tests for login flow
```

Mark dependencies and parallelizable work where relevant.
Callout important notes and hints in the "## Notes" section.

### 7. Validate

**Use Spox MCP tool:**

- `mcp__spox__validate_change` — Validate change proposal structure and content

Fix all issues before sharing.

## Output

When complete, you will have:

```
specs/_changes/<change-id>/
├── proposal.md           # Why + what + impact
├── tasks.md              # Implementation checklist
├── design.md             # (if needed) Technical decisions
└── specs/
    ├── <capability-1>/
    │   └── spec.md       # Delta specs for first affected capability
    ├── <capability-2>/
    │   └── spec.md       # Delta specs for second affected capability
    └── ...               # One folder per affected capability
```

**Do not implement.** Share the proposal and wait for approval.