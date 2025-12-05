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