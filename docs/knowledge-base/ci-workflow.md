# CI workflow

## Merge commit testing

GitHub Actions tests pull requests on the **merge commit**
of the PR branch with the base branch (typically `main`).
This means code that exists only on `main` — not on the PR
branch — is included in the test run.

When debugging a CI failure for a test that does not exist
on the PR branch, check whether the test was added to `main`
after the branch diverged.
