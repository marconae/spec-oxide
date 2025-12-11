# Spec Standards and Validation

Spec Oxide enforces structure and format standards for specs and changes.

## Spec File Format

Specs follow a standardized structure:

```markdown
# [capability] Specification

## Purpose

[Description of what this capability does and why it exists]

## Requirements

### Requirement: [Name]

[Normative description using SHALL or MUST]

#### Scenario: [Description]

- **WHEN** [condition]
- **THEN** [outcome]
- **AND** [additional outcome]

#### Scenario: [Another scenario]

- **WHEN** [condition]
- **THEN** [outcome]
```

**Key sections:**

| Section                        | Required | Description                            |
|--------------------------------|----------|----------------------------------------|
| `# [capability] Specification` | Yes      | Top-level heading with capability name |
| `## Purpose`                   | Yes      | Why this capability exists             |
| `## Requirements`              | Yes      | Container for all requirements         |

**Template location:** `.spox/templates/spec.md`

## Requirement Format

Requirements describe what the system SHALL or MUST do:

```markdown
### Requirement: [Name]

[Normative description using SHALL or MUST]
```

**Rules:**

- Use `### Requirement:` (h3 header)
- Include descriptive name after colon
- Use normative language: SHALL or MUST
- Provide clear description of the requirement

**Good examples:**

```markdown
### Requirement: User Authentication

The system SHALL authenticate users via username and password.

### Requirement: Password Validation

Passwords MUST be at least 8 characters long and contain at least one number.
```

**Bad examples:**

```markdown
### Requirement: Auth

Authentication stuff.

### User Login

The system authenticates users.
```

## Scenario Format

Scenarios describe concrete examples of requirement behavior:

```markdown
#### Scenario: [Description]

- **WHEN** [condition]
- **THEN** [outcome]
- **AND** [additional outcome]
```

**Rules:**

- Use `#### Scenario:` (h4 header, not bullets)
- Include descriptive name after colon
- Use WHEN for preconditions
- Use THEN for outcomes
- Use AND for additional outcomes (optional)

**Format matters:**

```markdown
#### Scenario: Valid credentials ✓ (h4 header)

- **Scenario: Valid credentials**   ✗ (bullet point)
  **Scenario**: Valid credentials ✗ (bold text)

### Scenario: Valid credentials ✗ (h3 header)
```

**Good example:**

```markdown
#### Scenario: Valid login credentials

- **WHEN** user provides valid username and password
- **THEN** system creates session token
- **AND** user is redirected to dashboard
```

**Bad examples:**

```markdown
- **Scenario: Valid login**
    - User logs in successfully

### Scenario: Login

Valid username and password → logged in
```

## Delta Format for Changes

Changes use delta specs to describe modifications:

```markdown
## ADDED Requirements

### Requirement: [Name]

[Description]

#### Scenario: [Description]

- **WHEN** [condition]
- **THEN** [outcome]

## MODIFIED Requirements

### Requirement: [Name]

[Complete updated requirement with all scenarios]

## REMOVED Requirements

### Requirement: [Name]

[Name only, no description needed]

## RENAMED Requirements

### Requirement: [Old Name] → [New Name]
```

**Operations:**

| Header                     | Use When                           |
|----------------------------|------------------------------------|
| `## ADDED Requirements`    | Adding new standalone capability   |
| `## MODIFIED Requirements` | Changing existing behavior         |
| `## REMOVED Requirements`  | Deprecating functionality          |
| `## RENAMED Requirements`  | Renaming without changing behavior |

**Critical for MODIFIED:**

When modifying a requirement, paste the **complete** existing requirement text, then edit. Partial text results in lost
content at archive time.

**Good MODIFIED example:**

```markdown
## MODIFIED Requirements

### Requirement: Password Authentication

Users SHALL authenticate using username and password OR email and password.

#### Scenario: Valid credentials with username

- **WHEN** valid username and password provided
- **THEN** user is logged in
- **AND** session token is created

#### Scenario: Valid credentials with email

- **WHEN** valid email and password provided
- **THEN** user is logged in
- **AND** session token is created
```

**Bad MODIFIED example:**

```markdown
## MODIFIED Requirements

### Requirement: Password Authentication

Users SHALL authenticate using username and password OR email.

[Missing original scenarios - content will be lost!]
```

## Scenario Syntax (WHEN/THEN/AND)

Scenarios use a structured format:

**WHEN clause (precondition):**

- Describes the triggering condition or action
- Always starts a scenario
- Required for every scenario

**THEN clause (outcome):**

- Describes the expected result
- Follows WHEN clause
- Required for every scenario

**AND clause (additional outcomes):**

- Describes additional expected results
- Follows THEN or another AND clause
- Optional but recommended

**Example:**

```markdown
#### Scenario: Successful file upload

- **WHEN** user selects file under 10MB
- **AND** file format is supported
- **THEN** file is uploaded to server
- **AND** upload progress is displayed
- **AND** success message is shown
```

## Validation Rules

Spec Oxide validates specs and changes for structural correctness.

### Spec Validation Rules

**Structure:**

- ✓ Must have `## Purpose` section
- ✓ Must have `## Requirements` section
- ⚠ Purpose section should be ≥50 characters

**Requirements:**

- ✓ Must use `### Requirement:` format
- ⚠ Should include descriptive text
- ⚠ Should use normative language (SHALL/MUST)
- ⚠ Should have at least one scenario

**Scenarios:**

- ✓ Must have WHEN clause
- ✓ Must have THEN clause
- ✓ Must use `#### Scenario:` format (h4 header)

### Change Validation Rules

**Proposal:**

- ✓ Must have `proposal.md` file
- ✓ Must have `## Why` section
- ✓ Must have `## What Changes` section
- ⚠ Why section should be ≥50 characters

**Tasks:**

- ✓ Must have `tasks.md` file
- ✓ Must contain at least one checkbox item (`- [ ]` or `- [x]`)
- ⚠ Task items should use numbered prefixes (e.g., `1.1`, `2.3.1`)

**Deltas:**

- ✓ Must have at least one delta spec
- ✓ Must use valid headers (ADDED, MODIFIED, REMOVED, RENAMED)
- ✓ Requirements must have at least one scenario
- ✓ Scenarios must have WHEN and THEN clauses

## Common Validation Errors

| Error                             | Cause                        | Fix                                                 |
|-----------------------------------|------------------------------|-----------------------------------------------------|
| "Missing Purpose section"         | No `## Purpose` in spec      | Add `## Purpose` section with description           |
| "Missing Requirements section"    | No `## Requirements` in spec | Add `## Requirements` section                       |
| "Missing proposal.md"             | Change lacks proposal        | Create `proposal.md` with Why/What Changes sections |
| "Missing tasks.md"                | Change lacks tasks           | Create `tasks.md` with checkbox items               |
| "Must have at least one delta"    | No delta specs in change     | Add `specs/<capability>/spec.md` with delta headers |
| "Must have at least one scenario" | Requirement has no scenarios | Add `#### Scenario:` with WHEN/THEN clauses         |
| "Scenario missing WHEN clause"    | Scenario lacks precondition  | Add `- **WHEN** [condition]` line                   |
| "Scenario missing THEN clause"    | Scenario lacks outcome       | Add `- **THEN** [outcome]` line                     |

## Validation Best Practices

**Before proposing:**

- Run `spox change validate` to catch issues early
- Ensure every requirement has at least one scenario
- Use normative language (SHALL/MUST) in requirements

**Before requesting approval:**

- Run `spox change validate` again
- Verify deltas use correct headers (ADDED/MODIFIED/REMOVED/RENAMED)
- Check scenario format (h4 headers, not bullets)

**After archiving:**

- Run `spox spec validate` to verify merged specs
- Confirm no validation errors or warnings
- Check that deltas were applied correctly

**In CI/CD:**

- Use `spox spec validate --strict` to fail on warnings
- Use `spox change validate --strict` to enforce high standards
- Run validation on every commit

## Troubleshooting Validation Issues

**"Silent parse failure" (validation passes but content missing):**

Run `spox change show <id> --deltas-only` to inspect parsed output. This shows exactly what the parser extracted.

**Common causes:**

- Wrong scenario format (bullets instead of h4 headers)
- Malformed delta headers
- Partial MODIFIED requirement text

**Validation passes but scenarios not detected:**

Check scenario format:

```markdown
#### Scenario: Name ✓

- **Scenario: Name**   ✗
```

**Validation fails with "Missing delta":**

Ensure `specs/_changes/<id>/specs/` directory contains `.md` files with valid delta headers:

```markdown
## ADDED Requirements

## MODIFIED Requirements

## REMOVED Requirements

## RENAMED Requirements
```