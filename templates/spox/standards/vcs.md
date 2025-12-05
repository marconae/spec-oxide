## Version Control Discipline

### Git Guardrails

**READ ONLY.** You may inspect git state. You must NEVER write to git.

#### Allowed (read-only)
```bash
git status [--short]    # Working tree state
git diff [--staged]     # View changes
git log [--oneline -n]  # Commit history
git show                # Commit details
git branch [-a|-r]      # List branches
```

#### FORBIDDEN — Never Execute

**Any command that modifies repository state is prohibited:**

- `git add`, `git commit`, `git push`, `git pull`, `git fetch`
- `git merge`, `git rebase`, `git cherry-pick`
- `git checkout`, `git switch`, `git restore`
- `git reset`, `git revert`
- `git stash` ← **including stash; do not use**
- `git tag`, `git remote add/remove`, `git submodule`

**No exceptions. No workarounds. User controls all git writes.**

#### Before Completing Work
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