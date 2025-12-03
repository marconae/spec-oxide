# validation Specification

## Purpose

Define the validation system for Spec Oxide that verifies spec structure (Purpose/Requirements sections), requirement
format (normative language, scenarios), scenario format (WHEN/THEN clauses), change proposals, delta specs, and tasks.md
files.

## Requirements

### Requirement: Spec Structure Validation

The validation system SHALL verify that spec files follow the required structure.

#### Scenario: Valid spec structure

- **WHEN** a spec file contains `## Purpose` and `## Requirements` sections
- **THEN** structural validation passes
- **AND** no structural errors are reported

#### Scenario: Missing Purpose section

- **WHEN** a spec file lacks a `## Purpose` section
- **THEN** validation reports an ERROR
- **AND** the error message indicates "Missing Purpose section"
- **AND** the error includes the file path

#### Scenario: Missing Requirements section

- **WHEN** a spec file lacks a `## Requirements` section
- **THEN** validation reports an ERROR
- **AND** the error message indicates "Missing Requirements section"

#### Scenario: Short Purpose warning

- **WHEN** a spec file has a Purpose section with fewer than 50 characters
- **THEN** validation reports a WARNING
- **AND** the warning suggests adding more detail

### Requirement: Requirement Block Validation

The validation system SHALL verify that requirement blocks follow the correct format.

#### Scenario: Valid requirement format

- **WHEN** a requirement block has `### Requirement: <name>` header
- **AND** contains descriptive text with SHALL or MUST
- **AND** has at least one `#### Scenario:` block
- **THEN** validation passes for that requirement

#### Scenario: Missing requirement text

- **WHEN** a requirement block has only a header and scenarios
- **THEN** validation reports a WARNING
- **AND** the warning indicates the requirement lacks descriptive text

#### Scenario: Missing normative language

- **WHEN** a requirement block text does not contain SHALL or MUST
- **THEN** validation reports a WARNING
- **AND** the warning suggests using normative language

#### Scenario: Missing scenarios

- **WHEN** a requirement block has no `#### Scenario:` sections
- **THEN** validation reports a WARNING
- **AND** the warning indicates the requirement needs scenarios

### Requirement: Scenario Format Validation

The validation system SHALL verify that scenarios follow the WHEN/THEN format.

#### Scenario: Valid scenario format

- **WHEN** a scenario contains a WHEN clause and a THEN clause
- **THEN** validation passes for that scenario

#### Scenario: Missing WHEN clause

- **WHEN** a scenario lacks a WHEN clause
- **THEN** validation reports an ERROR
- **AND** the error indicates "Scenario missing WHEN clause"
- **AND** the error includes the scenario name and line number

#### Scenario: Missing THEN clause

- **WHEN** a scenario lacks a THEN clause
- **THEN** validation reports an ERROR
- **AND** the error indicates "Scenario missing THEN clause"

### Requirement: Change Proposal Validation

The validation system SHALL verify that change proposals follow the required structure.

#### Scenario: Valid proposal structure

- **WHEN** a change has `proposal.md` with `## Why` and `## What Changes` sections
- **THEN** structural validation passes

#### Scenario: Missing proposal.md

- **WHEN** a change directory lacks `proposal.md`
- **THEN** validation reports an ERROR
- **AND** the error indicates "Missing proposal.md"

#### Scenario: Missing Why section

- **WHEN** `proposal.md` lacks a `## Why` section
- **THEN** validation reports an ERROR
- **AND** the error indicates "Missing Why section"

#### Scenario: Short Why section

- **WHEN** the Why section has fewer than 50 characters
- **THEN** validation reports a WARNING

#### Scenario: Missing What Changes section

- **WHEN** `proposal.md` lacks a `## What Changes` section
- **THEN** validation reports an ERROR
- **AND** the error indicates "Missing What Changes section"

### Requirement: Delta Spec Validation

The validation system SHALL verify that delta specs use correct operation headers.

#### Scenario: Valid delta headers

- **WHEN** a delta spec uses `## ADDED Requirements`, `## MODIFIED Requirements`, `## REMOVED Requirements`, or
  `## RENAMED Requirements`
- **THEN** the delta header is recognized
- **AND** the section is parsed for requirements

#### Scenario: At least one delta required

- **WHEN** a change has no delta specs in its `specs/` directory
- **THEN** validation reports an ERROR
- **AND** the error indicates "Change must have at least one delta"

#### Scenario: ADDED requirement validated

- **WHEN** a delta has `## ADDED Requirements` section
- **THEN** each requirement under it is validated as a new requirement
- **AND** scenarios are checked for WHEN/THEN format

#### Scenario: MODIFIED requirement validated

- **WHEN** a delta has `## MODIFIED Requirements` section
- **THEN** each requirement under it is validated for completeness
- **AND** a WARNING is reported if the requirement text appears incomplete

#### Scenario: REMOVED requirement acknowledged

- **WHEN** a delta has `## REMOVED Requirements` section
- **THEN** the removal is recorded
- **AND** no structural validation is performed on removed items

### Requirement: Validation Report Output

The validation system SHALL produce clear, actionable output.

#### Scenario: Error output format

- **WHEN** validation finds an ERROR
- **THEN** output includes severity level (ERROR)
- **AND** output includes file path and line number when available
- **AND** output includes a descriptive message

#### Scenario: Summary output

- **WHEN** validation completes
- **THEN** output includes a summary line with error and warning counts
- **AND** the summary appears after all individual issues

#### Scenario: Exit code reflects validity

- **WHEN** validation finds no errors
- **THEN** the command exits with code 0

#### Scenario: Exit code reflects errors

- **WHEN** validation finds one or more errors
- **THEN** the command exits with code 1

### Requirement: Strict Mode Validation

The validation system SHALL support a strict mode for CI/CD pipelines.

#### Scenario: Strict mode enabled

- **WHEN** `--strict` flag is provided
- **THEN** both errors AND warnings cause validation to fail
- **AND** exit code is 1 if any warnings exist

#### Scenario: Normal mode

- **WHEN** `--strict` flag is not provided
- **THEN** only errors cause validation to fail
- **AND** warnings are reported but do not affect exit code

### Requirement: Tasks File Validation

The validation system SHALL verify that tasks.md files follow the required checklist format. This is a Spec Oxide
feature not present in OpenSpec.

#### Scenario: Valid tasks structure

- **WHEN** a change has `tasks.md` with numbered task groups and checkbox items
- **THEN** structural validation passes

#### Scenario: Missing tasks.md

- **WHEN** a change directory lacks `tasks.md`
- **THEN** validation reports an ERROR
- **AND** the error indicates "Missing tasks.md"

#### Scenario: No task items found

- **WHEN** `tasks.md` contains no checkbox items (`- [ ]` or `- [x]`)
- **THEN** validation reports an ERROR
- **AND** the error indicates "tasks.md must contain at least one task item"

#### Scenario: Task numbering format

- **WHEN** task items use numbered prefixes (e.g., `1.1`, `2.3.1`)
- **THEN** validation passes
- **AND** the numbering is recognized as valid

#### Scenario: Unnumbered tasks warning

- **WHEN** task items lack numbered prefixes
- **THEN** validation reports a WARNING
- **AND** the warning suggests using numbered prefixes for traceability

#### Scenario: Task group headers

- **WHEN** `tasks.md` has `## N. Group Name` headers (e.g., `## 1. Foundation`)
- **THEN** the groups are recognized as valid structure
- **AND** tasks under each group are associated with that group

#### Scenario: Tasks completion tracking

- **WHEN** validating a change with tasks
- **THEN** the validation report includes task completion statistics
- **AND** shows count of completed (`- [x]`) vs pending (`- [ ]`) tasks

#### Scenario: Nested subtasks supported

- **WHEN** task items have indented sub-items with checkboxes
- **THEN** the subtasks are recognized and counted
- **AND** subtask completion is included in statistics

