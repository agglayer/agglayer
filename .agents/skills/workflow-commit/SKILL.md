---
name: commit
description: Create a git commit following project conventions.
disable-model-invocation: true
---

When creating a git commit:
- Use Conventional Commits format (see [Format](#format))
- Follow repository commit conventions

## Format

```
<type>(<optional-scope>): <description>

[optional body]

[optional footer(s)]
```

## Rules

- **Subject line**: imperative mood, lowercase start, no trailing period.
- **Subject line max**: 72 characters.
- **Body lines max**: 72 characters,
  except long URLs, code blocks, or stack traces.
- **Allowed types**: `feat`, `fix`, `docs`, `chore`, `style`,
  `refactor`, `perf`, `test`, `build`, `ci`, `revert`.
- **Scope**: optional, should name the affected crate or area
  (e.g., `prover`, `grpc`, `config`).
- **Footer**: `CONFIG-CHANGE:` for configuration changes (multi-line allowed).
  `BREAKING-CHANGE:` for breaking changes (multi-line allowed).

## Steps

1. Review staged changes with `git diff --cached`.
2. Draft a commit message following the rules above.
3. Present the message for approval before committing.
4. Create the commit.
   Do not push unless explicitly asked.

## Additional resources

- For complete convention details, see [conventional-commits](https://conventionalcommits.org/en/v1.0.0/)
- For usage examples, see [samples.md](examples/samples.md)
