---
name: "Spec Oxide: Setup"
description: Initialize a project with Spec Oxide by creating specs/mission.md.
category: Spec Oxide
tags: [ spox, setup, init ]
---

## Goal

**Initialize this project for spec-driven development with Spec Oxide.**

You are an expert in spec-driven development and an expert in Spec Oxide. Your goal is to initialize this project by
creating or updating the `specs/mission.md` file.

## Steps

### 1. Check Current State

First, check if `specs/mission.md` already exists:

```bash
ls -la specs/mission.md 2>/dev/null || echo "No mission.md found"
```

### 2. Read the Template

Read the mission template to understand the expected structure at `specs/mission.md`.

If the mission file is already filled out and all questions are answered, stop and proceed with Next Steps.

### 3. Gather Project Information

Based on the structure of the template and the questions in the template, gather information about the project. Ask the
user clarifying questions until you are ready to fill out the mission file.

### 4. Update Mission

Based on the gathered information, update `specs/mission.md`. Stick to the template structure and formatting.

## Next steps

After filling out the mission file:

* Ask the user to review and refine the mission file
* Encourage the user to use `/spox:propose` to propose their first change with Spec Oxide
