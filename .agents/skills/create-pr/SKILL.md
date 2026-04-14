---
name: create-pr
description: Create a pull request following project conventions.
disable-model-invocation: true
---

When creating a PR:

- Use `gh pr create`
- Use the target repo's dedicated PR template when one exists.
- Otherwise, use `pull_request_template.md`
  from this package as the fallback template.
- Follow merge-queue conventions.

## Merge-queue mapping

- **PR title** = commit title (Conventional Commits format).
- **PR description** = commit body (reusable as commit message content).
- **PR title max**: 72 characters.
- **PR description line max**: 72 characters,
  except long URLs, code blocks, or stack traces.

## Description rules

- The **first line** must be plain context text describing the change.
- After the context line, follow the selected template.
- If the target repo has a dedicated PR template, follow that template
  after the context line.
- Otherwise, append the sections from
  `pull_request_template.md` after the context line.
- Do not add extra headings beyond those required by the selected template.
- Fill `CONFIG-CHANGE:` and `BREAKING-CHANGE:` sections when applicable.
- **Remove** `CONFIG-CHANGE:` and/or `BREAKING-CHANGE:` sections entirely
  if there is no config change or breaking change respectively.
- Keep the description concise and commit-friendly.

## Steps

1. **Run the `verify` skill** if it has not been run
   since the last code change.
   All matching checks must pass before proceeding.
2. Review all commits on the branch
   (`git log` and `git diff` against the base branch).
3. Draft a PR title (Conventional Commits format, 72 chars max).
4. Draft a PR description that starts with a plain context line.
5. If the target repo has a dedicated PR template, follow that template
   after the context line; otherwise append
   `pull_request_template.md` after the context line.
6. Ask for confirmation from the user with all this information.
7. Check if the branch needs to be pushed to remote.
8. Create the PR.
9. Return the PR URL.
