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

```bash
spox config show      # Get authoritative paths
spox spec list        # See existing capabilities
spox change list      # Check for in-progress changes that might conflict
```

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
* `.spox/specs/change/proposal.md`
* `.spox/specs/change/tasks.md`
* `.spox/specs/change/design.md` (if needed)
* `.spox/specs/change/spec.md`

### 4. Write Spec Deltas

Place deltas in `specs/_changes/<change-id>/specs/<capability>/spec.md`.

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

### 5. Write Tasks

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

### 6. Validate

```bash
spox change validate
```

Fix all issues before sharing.

## Output

When complete, you will have:

```
specs/_changes/<change-id>/
├── proposal.md           # Why + what + impact
├── tasks.md              # Implementation checklist  
├── design.md             # (if needed) Technical decisions
└── specs/
    └── <capability>/
        └── spec.md       # Delta specs with scenarios
```

**Do not implement.** Share the proposal and wait for approval.

## Quick Reference

```bash
spox change show <id>                   # View your proposal
spox change show <id> --deltas-only     # Debug parsing issues
rg -n "Requirement:" specs              # Search existing requirements
```