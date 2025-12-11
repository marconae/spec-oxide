# Workflow

## Just start in under 2 minutes

```bash
# Initialize a new project
spox init

# Run the setup script to configure MCP servers (Serena, Context7)
.spox/setup.sh

# Run Claude Code
claude

# Get started with /spox:setup
```

## Overview

AI coding assistants are powerful but unpredictable when requirements live only in chat history. Spec Oxide solves this
by:

1. **Separating intent from implementation** - Define what to build before writing code
2. **Creating an audit trail** - Every change is documented and reviewable
3. **Enabling deterministic outputs** - AI agents work from specs, not vague prompts
4. **Maintaining up-to-date documentation** - Specs evolve as the project grows

Spec Oxide follows a three-stage workflow: **Propose → Implement → Archive**.

<img src="assets/spox-overview.svg" alt="Spec Oxide Workflow" width="700">

## The Three Stages

### Stage 1: Propose

**Goal:** Agree on what to build before writing any code.

Use the `/spox:propose` slash command in Claude Code to create a change proposal:

```
/spox:propose Add two-factor authentication to the login flow
```

This creates a structured proposal in `specs/_changes/<change-id>/`:

```
specs/_changes/add-two-factor-auth/
  proposal.md           # Why this change? What's the impact?
  tasks.md              # Ordered implementation checklist
  design.md             # Technical decisions (optional)
  specs/
    auth/
      spec.md           # Delta requirements
```

The command is designed to discuss any open questions or ambiguities with you. Read and discuss the `proposal.md`,
`design.md`, `tasks.md`, and drafted change delta `specs/<capability>` folders.

Approve the proposal by instructing the agent to implement the task list with `/spox:implement`.

### Stage 2: Implement

Use the `/spox:implement` slash command to execute the approved change:

```
/spox:implement add-two-factor-auth
```

This orchestrates implementation by:

1. Reading the proposal, design, and tasks
2. Spawning subagents to complete tasks
3. Tracking progress in `tasks.md`
4. Verifying implementation against specs

**Key activities:**

- Work through tasks systematically
- Mark completed tasks in `tasks.md`
- Run verification before marking complete
- Ensure implementation matches the spec exactly

**Do not archive.** Wait for deployment before moving to Stage 3.

### Stage 3: Archive

**Goal:** Merge approved changes into the source of truth.

After deployment, use the `/spox:archive` slash command:

```
/spox:archive add-two-factor-auth
```

This finalizes the change by:

1. Moving `specs/_changes/<id>/` → `specs/_archive/YYYY-MM-DD-<id>/`
2. Applying deltas to `specs/<capability>/spec.md` files
3. Validating specs: `spox spec validate`

**Key activities:**

- Move change to archive with timestamp
- Apply ADDED/MODIFIED/REMOVED/RENAMED deltas to specs
- Verify specs are consistent
- Confirm the source of truth is updated

## Slash Commands Overview

Spec Oxide integrates with Claude Code via slash commands:

| Command           | Purpose                              | Stage     |
|-------------------|--------------------------------------|-----------|
| `/spox:setup`     | Initialize project mission           | Setup     |
| `/spox:propose`   | Create change proposals              | Propose   |
| `/spox:implement` | Implement approved changes           | Implement |
| `/spox:archive`   | Archive completed changes            | Archive   |
| `/spox:vibe`      | Vibe coding mode (rapid prototyping) | N/A       |

**When to use each command:**

- **`/spox:setup`** - After `spox init`, to define your project mission in `specs/mission.md`
- **`/spox:propose`** - When you have a new feature, breaking change, or architectural update
- **`/spox:implement`** - When a proposal has been reviewed and approved
- **`/spox:archive`** - After implementation is deployed to production
- **`/spox:vibe`** - For quick experiments or throwaway prototypes outside the spec workflow

## Example Workflow Walkthrough

Let's walk through adding two-factor authentication:

**1. Propose**

```bash
# Check current state
spox spec list
spox change list

# Create proposal
/spox:propose Add two-factor authentication to login flow
```

The AI agent creates:

- `specs/_changes/add-two-factor-auth/proposal.md` - Why and what changes
- `specs/_changes/add-two-factor-auth/tasks.md` - Implementation checklist
- `specs/_changes/add-two-factor-auth/specs/auth/spec.md` - Delta requirements

```bash
# Validate proposal
spox change validate add-two-factor-auth
```

Review the proposal, approve it, and move to implementation.

**2. Implement**

```bash
# Implement the change
/spox:implement add-two-factor-auth
```

The AI agent:

- Reads the proposal and specs
- Works through tasks in `tasks.md`
- Marks completed tasks with `[x]`
- Runs verification

```bash
# Check progress
spox change show add-two-factor-auth
```

Deploy to production after verification passes.

**3. Archive**

```bash
# Archive the change
/spox:archive add-two-factor-auth
```

The AI agent:

- Moves change to `specs/_archive/2025-12-06-add-two-factor-auth/`
- Applies deltas to `specs/auth/spec.md`
- Validates specs

```bash
# Verify specs
spox spec validate
spox spec list
```

Done! The two-factor authentication capability is now part of the permanent specification.