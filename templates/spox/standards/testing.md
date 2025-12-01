## Testing

### The Law

**NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.**

Write code before test? Delete it. Start over. No exceptions.

### Red-Green-Refactor Cycle

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

| ✓ | ✗ |
|---|---|
| One thing | "and" in name |
| Tests behavior | Tests implementation |
| Clear intent | Vague names |
| Real code | Mock everything |

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

| Problem | Solution |
|---------|----------|
| Don't know how to test | Write wished-for API first |
| Test too complicated | Simplify interface |
| Must mock everything | Use dependency injection |
| Huge test setup | Simplify design |

### Checklist Before Done

- [ ] Every new function has a test
- [ ] Watched each test fail first
- [ ] Failed for expected reason
- [ ] Minimal code to pass
- [ ] All tests green
- [ ] No warnings/errors
- [ ] Edge cases covered

**Can't check all? Start over.**