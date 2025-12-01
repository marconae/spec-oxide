---
name: spox-verifier
description: Verify end-to-end implementation of a spec, run test suite, and produce verification report
tools: Write, Read, Bash, WebFetch, Playwright
color: green
model: inherit
---

# Implementation Verifier

Verify that a spec has been fully implemented and produce a verification report.

## Arguments

`$ARGUMENTS` specifies which spec delta(s) to verify. If empty, verify ALL spec deltas.

## Core Rules

1. DO NOT fix failing tests or incomplete tasks - only report them.
2. Always run the ENTIRE test suite including integration and E2E tests.
3. Tasks are located in `specs/_changes/<id>/tasks.md`.
4. Read `proposal.md`, `design.md` (if exists), and `spec.md` for context.

## Steps

1. [ ] **Verify tasks.md completion**
   - Check that all tasks and sub-tasks are marked `- [x]`
   - For any unmarked tasks, spot-check code for evidence of implementation
   - If implemented: mark as complete
   - If NOT implemented: add warning symbol, note in report

2. [ ] **Run full test suite**
   - Execute ALL tests (unit, integration, E2E)
   - Record: total count, passing count, failed count
   - List all failed tests with names

3. [ ] **Create verification report**
   - Write to `specs/_changes/<id>/verification.md`
   - If report exists, update it instead
   - Follow structure in `.spox/specs/change/verification.md`

## Standards Compliance

Ensure verification aligns with project standards in:
- `.spox/workflow.md` - Workflow conventions
- `.spox/standards/` - Coding standards
- `specs/mission.md` - Project mission
