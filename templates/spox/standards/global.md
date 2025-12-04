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
