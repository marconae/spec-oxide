# workflow Specification

## Purpose

Define the Claude Code slash commands that guide users through the Spec Oxide workflow stages: propose, implement, archive, and setup.

## Requirements

### Requirement: Setup Slash Command

The workflow SHALL provide a `/spox:setup` slash command that guides users through project initialization.

#### Scenario: Initialize new project

- **WHEN** user invokes `/spox:setup`
- **THEN** Claude SHALL read the existing `specs/mission.md` template
- **AND** help the user fill it out with project-specific details including tech stack and conventions
