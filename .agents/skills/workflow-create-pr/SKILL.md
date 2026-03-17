---
name: create-pr
description: Create a pull request following project conventions.
disable-model-invocation: true
---

When creating a PR:
- Use `gh pr create`
- Use the project's template and merge-queue conventions.

## Merge-queue mapping

- **PR title** = commit title (Conventional Commits format).
- **PR description** = commit body (reusable as commit message content).
- **PR title max**: 72 characters.
- **PR description line max**: 72 characters,
  except long URLs, code blocks, or stack traces.

## Description rules

- **No headings** (no `## Summary` or similar) in the PR description.
- The **first line** must be plain context text describing the change.
- Fill `CONFIG-CHANGE:` and `BREAKING-CHANGE:` sections when applicable.
- **Remove** `CONFIG-CHANGE:` and/or `BREAKING-CHANGE:` sections entirely
  if there is no config change or breaking change respectively.
- Keep the description concise and commit-friendly.

## Steps

1. Review all commits on the branch
   (`git log` and `git diff` against the base branch).
2. Draft a PR title (Conventional Commits format, 72 chars max).
3. Draft a PR description following the rules above.
4. Ask for confirmation from the user with all this information.
5. Check if the branch needs to be pushed to remote.
6. Create the PR.
7. Return the PR URL.
