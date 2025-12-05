<!-- SPOX:START -->
# Standards Compliance and Guardrails

## MCP Tools

Prefer MCP tools over text-based alternatives. MCP tools provide semantic understanding; text tools only match patterns.

### Serena MCP

**Use for:** Code understanding, code navigation, symbol lookup, reference finding, code analysis.

**Prefer over:** `rg`, `grep`, `find`, `ag`, `ast-grep`, or any text-based search.

| Task                      | Use Serena                | Not                    |
|---------------------------|---------------------------|------------------------|
| Find function definition  | `serena.find_definition`  | `rg "function foo"`    |
| Find all references       | `serena.find_references`  | `rg "foo("`            |
| Understand code structure | `serena.get_symbols`      | `rg "class\|function"` |
| Navigate to symbol        | `serena.go_to_symbol`     | `grep -rn "symbol"`    |
| Analyze dependencies      | `serena.get_dependencies` | Manual file reading    |

**Why:** Serena understands code semantically. It knows that `foo()` in file A calls the `foo` defined in file B. Text
search only finds string matches.

### Context7 MCP

**Use for:** Library documentation, API references, framework guides, package usage.

**Prefer over:** Training data, assumptions, outdated memory, web search for docs.

| Task                          | Use Context7        | Not                        |
|-------------------------------|---------------------|----------------------------|
| Check API signature           | `context7.lookup`   | Assume from training       |
| Find library examples         | `context7.examples` | Guess syntax               |
| Verify current behavior       | `context7.docs`     | Rely on outdated knowledge |
| Understand framework patterns | `context7.guides`   | Web search                 |

**Why:** Training data has a cutoff date. Libraries change. Context7 provides current, accurate documentation. When in
doubt, verify with Context7.

## Decision Flow

```
Need to understand code?
├─ Symbol, reference, or structure → Serena
├─ Library or API docs → Context7
└─ Neither available → Fall back to text tools
```

### Rules

1. **Serena first** for any code navigation or understanding task
2. **Context7 first** for any library or documentation lookup
3. **Text tools** only when MCP tools are unavailable or insufficient
4. **Never assume** library behavior—verify with Context7
5. **Never pattern-match** code—understand it with Serena

## General mindset

- Consistent, discoverable directory structure
- README docs kept current
- Environment variables for config; never commit secrets
- Minimal dependencies; justify additions
- Feature flags over long-running branches
- Changelog for notable changes

## Simplicity First
- Default to <100 lines of new code
- Single-file implementations until proven insufficient
- Avoid frameworks without clear justification
- Choose boring, proven patterns

## Verification

Adhere to the following rules:
- Use Test Driven Development (TDD)
- **Always plan and implement** Unit Tests
- Aim for >80% code coverage with Unit Tests (both lines and branches)
- **Always plan and implement** Integration Tests
- Integration Tests SHALL cover requirements and scenarios

## Complexity Triggers
Only add complexity with:
- Performance data showing current solution too slow
- Concrete scale requirements (>1000 users, >100MB data)
- Multiple proven use cases requiring abstraction

## Error handling best practices

**User-facing:** Actionable messages, no internal details

**Code patterns:**
- Fail fast: validate early, reject invalid state
- Specific error types for targeted catch blocks
- Centralize handling at boundaries (API/controller layer)
- Graceful degradation for non-critical failures
- Exponential backoff for transient external failures
- Always release resources (finally/defer/using)

## Coding Standards

### Core Principles

1. **KISS** Keep it simple stupid. Simpler is always better. Reduce complexity as much as possible.
2. **Boy scout rule**. Leave the campground cleaner than you found it.
3. **Always find root cause**. Always look for the root cause of a problem.
4. **DRY**: Eliminate duplication. Extract shared logic instead of copy-pasting.
5. **YAGNI**: Build only what is required now. Add complexity only when evidence demands it.
6. **Single Responsibility**: One function, one purpose. If it needs “and” to describe it, split it.

### Design
1. Keep configurable data at high levels.
2. Prefer polymorphism over if/else or switch/case.
3. Isolate multi-threading code.
4. Avoid over-configurability.
5. Use dependency injection.
6. Follow Law of Demeter—classes know only direct dependencies.

### Understandability
1. Be consistent in style and approach.
2. Use explanatory variables.
3. Encapsulate boundary conditions in one place.
4. Prefer value objects over primitives.
5. Avoid logical dependencies within a class.
6. Avoid negative conditionals.

### Naming
1. Choose descriptive, unambiguous names.
2. Make meaningful distinctions.
3. Use pronounceable, searchable names.
4. Replace magic numbers with named constants.
5. Avoid prefixes and type encodings.

### Functions
1. Keep small.
2. Do one thing.
3. Use descriptive names.
4. Minimize arguments.
5. Avoid side effects.
6. No flag arguments—split into separate methods.

### Comments
1. Prefer self-documenting code.
2. Avoid redundancy and obvious noise.
3. No closing brace comments.
4. Delete commented-out code.
5. Use only for intent, clarification, or warnings.
6. Never use comments to track work.

### Structure
1. Separate concepts vertically.
2. Keep related code vertically dense.
3. Declare variables near usage.
4. Place dependent and similar functions close together.
5. Order functions top-down by call hierarchy.
6. Keep lines short; avoid horizontal alignment.
7. Use whitespace to show relationships.
8. Maintain consistent indentation.

### Objects & Data Structures
1. Hide internal structure.
2. Prefer pure data structures or objects—avoid hybrids.
3. Keep classes small, focused, with few instance variables.
4. Base classes shouldn't know their derivatives.
5. Prefer many small functions over flag-based behavior selection.
6. Prefer instance methods over static methods.

### Code Smells
1. **Rigidity**: Small changes cascade.
2. **Fragility**: Single changes break many places.
3. **Immobility**: Code not reusable elsewhere.
4. **Needless complexity**.
5. **Needless repetition**.
6. **Opacity**: Hard to understand.

### Code Organization

**Imports**: Standard → third-party → local. Remove unused imports.

**Dead Code**: Delete unused or commented-out code. Rely on version control for history.

**Function Size**: Keep functions small. Extract complex logic into helpers.

## Testing

### Guardrails for Testing

- You have to use Test Driven Development (TDD).
- **NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.**
- Write code before test? Delete it. Start over. No exceptions.

### Red-Green-Refactor Cycle

Apply the Red-Green-Refactor cycle to every new feature or bug fix:

```
RED → verify fails → GREEN → verify passes → REFACTOR → repeat
```

#### RED: Write Failing Test

- One behavior per test
- Clear descriptive name
- Real code, minimal mocks

#### Verify RED (MANDATORY)

```bash
npm test path/to/test.test.ts  # or equivalent
```

- Must fail (not error)
- Fails because feature missing, not typos
- Test passes? Fix the test.

#### GREEN: Minimal Code

- Simplest code to pass
- No extra features
- No "improvements"

#### Verify GREEN (MANDATORY)

- Test passes
- All other tests pass
- No warnings/errors

#### REFACTOR

- Only after green
- Remove duplication, improve names
- Keep tests green
- Don't add behavior

### Good Tests

| ✓              | ✗                    |
|----------------|----------------------|
| One thing      | "and" in name        |
| Tests behavior | Tests implementation |
| Clear intent   | Vague names          |
| Real code      | Mock everything      |

### Bug Fixes

1. Write failing test reproducing bug
2. Follow TDD cycle
3. Never fix without a test

### Red Flags → DELETE & RESTART

- Code before test
- Test passes immediately
- "I'll test after"
- "Just this once"
- "Keep as reference"
- "Already manually tested"
- "Too simple to test"

### When Stuck

| Problem                | Solution                   |
|------------------------|----------------------------|
| Don't know how to test | Write wished-for API first |
| Test too complicated   | Simplify interface         |
| Must mock everything   | Use dependency injection   |
| Huge test setup        | Simplify design            |

### Checklist Before Done

- [ ] Every new function has a test
- [ ] Watched each test fail first
- [ ] Failed for expected reason
- [ ] Minimal code to pass
- [ ] All tests green
- [ ] No warnings/errors
- [ ] Edge cases covered

**Can't check all? Start over.**

## Backend

### API endpoint standards and conventions

`{resources}` is exemplary for the endpoints to implement:

```
GET    /{resources}      → list
GET    /{resources}/:id  → show
POST   /{resources}      → create
PUT    /{resources}/:id  → replace
PATCH  /{resources}/:id  → update
DELETE /{resources}/:id  → destroy
```

**URL Rules:**
- Plural nouns: `/users`, `/products`
- Lowercase, hyphenated: `/user-profiles`
- Max 2-3 nesting levels: `/users/:id/orders`
- Query params for filtering/sorting/pagination

**Responses:**
- 200 OK, 201 Created, 204 No Content
- 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found
- 500 Internal Server Error

**Headers:** Include rate limit info (`X-RateLimit-Limit`, `X-RateLimit-Remaining`)

**Versioning:** Use `/v1/` prefix or `Accept` header

### Database migration best practices

**Every migration must:**
- Have a working rollback/down method
- Make one logical change only
- Use descriptive name: `add_email_index_to_users`

**For production:**
- Schema changes separate from data migrations
- Concurrent index creation on large tables
- Test rollback before deploying
- Consider backwards compatibility for zero-downtime

### Database model best practices

**Required on all tables:**
- `created_at`, `updated_at` timestamps
- Primary key (prefer UUID or auto-increment)

**Constraints:**
- NOT NULL where data is required
- UNIQUE for natural keys
- Foreign keys with appropriate CASCADE

**Indexes:**
- All foreign key columns
- Frequently filtered/sorted columns
- Composite indexes for common query patterns

**Naming:**
- Models: singular (`User`)
- Tables: plural (`users`)
- Foreign keys: `{table}_id`

### Database query best practices

**Security (example representative for all languages):**
```python
# ✓ Parameterized
db.query("SELECT * FROM users WHERE id = ?", [user_id])

# ✗ NEVER interpolate
db.query(f"SELECT * FROM users WHERE id = {user_id}")
```

**Performance:**
- Select specific columns, not `SELECT *`
- Only select what is required
- Eager load relations to prevent N+1
- Prefer joins for eager loading, avoid subqueries
- Use transactions for related writes
- Set query timeouts
- Cache expensive queries

**Indexes:** Add to columns in WHERE, JOIN, ORDER BY clauses

## UI Standards

### UI accessibility best practices

**Required:**
- Semantic HTML (`<nav>`, `<main>`, `<button>`, not `<div>` for everything)
- Keyboard navigable with visible focus states
- Alt text on images, labels on form inputs
- Heading hierarchy: h1 → h2 → h3 (no skipping)

**Color:** 4.5:1 contrast minimum; never color-only information

**ARIA:** Only when semantic HTML insufficient

**Dynamic content:** Manage focus on modals, route changes, live updates

**Verify:** Test with screen reader before shipping

### UI component best practices

**Design principles:**
- Single responsibility (one purpose per component)
- Composable (combine small components, not monoliths)
- Reusable (configurable via props)
- Encapsulated (hide internals, expose minimal API)

**Props:**
- Explicit types with sensible defaults
- Keep count low; many props → split component
- Document with examples

**State:** Keep local; lift only when shared

**Naming:** Descriptive, consistent with project conventions

### CSS best practices

**Rules:**
- Follow project methodology (Tailwind/BEM/modules) consistently
- Use design tokens (colors, spacing, typography)
- Work with framework patterns, don't override
- Minimize custom CSS

**Production:** Enable purging/tree-shaking for unused styles

### Responsive design best practices

**Approach:** Mobile-first, progressive enhancement to large screens

**Layout:**
- Fluid containers (%, fr)
- Relative units (rem/em over px)
- Standard breakpoints (mobile → tablet → desktop)

**Touch:** Minimum 44×44px tap targets

**Typography:** Readable at all breakpoints without zoom

**Performance:** Optimize images/assets for mobile

**Verify:** Test on real devices across breakpoints

## Version Control Discipline

### Git Guardrails

- **You may READ git state, but you must NEVER WRITE to git.**
- Writing to Git is fully controlled by the user.

#### Commands

**Allowed Commands** Use the following only to inspect changes, branch, commits, conflicts, and overall repo state:
```bash
git status              # Check working tree
git status --short      # Compact status
git diff                # Unstaged changes
git diff --staged       # Staged changes
git diff HEAD~1         # Compare with previous commit
git log                 # Commit history
git log --oneline -10   # Recent commits
git show <commit>       # Commit details
git branch              # Local branches
git branch -a           # All branches
git branch -r           # Remote branches
```

**NEVER execute these commands under any circumstances:**

```bash
git add                 # Staging
git commit              # Committing
git push                # Pushing
git pull                # Pulling
git fetch               # Fetching
git merge               # Merging
git rebase              # Rebasing
git checkout            # Switching branches/files
git switch              # Switching branches
git restore             # Restoring files
git reset               # Resetting
git revert              # Reverting
git stash               # Stashing
git cherry-pick         # Cherry-picking
git tag                 # Tagging
git remote add/remove   # Remote management
git submodule           # Submodule operations
```

#### Verify status before completing work

**Mandatory** Check git status before marking work complete:

```bash
git status              # Verify expected files changed
git diff                # Review actual changes
```

### Commit Conventions

The commit message SHALL be structured like this:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Use conventional commits types:

- `feat:` for new features (correlates with MINOR in Semantic Versioning)
- `fix:` for bug fixes (correlates with PATCH in Semantic Versioning)
- `perf:` for performance improvements
- `refactor:` for code restructuring
- `test:` for test additions/changes
- `docs:` for documentation
- `spec` for spec changes
- `chore:` for maintenance tasks

BREAKING CHANGE: a commit that has a footer BREAKING CHANGE:, or appends a ! after the type/scope, introduces a breaking
API change (correlating with MAJOR in Semantic Versioning). A BREAKING CHANGE can be part of commits of any type.


# Spec Oxide Workflow

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

**CLI:** Spec Oxide uses a CLI tool called `spox` to list, view, and validate specs and changes.

**Template for specs:**

Use this template for spec documents for the current truth: `.spox/templates/spec.md`

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

# Project Mission

Read and understand the project mission:
@specs/mission.md

<!-- SPOX:END -->