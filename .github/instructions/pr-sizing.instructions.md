---
applyTo: "**"
---

When reviewing a pull request, classify its size by **review effort and semantic complexity**, not raw line count.

Use these labels: `size/xs`, `size/s`, `size/m`, `size/l`, `size/xl`.

Discount mechanical churn when it does not add real review cost:
- lockfiles
- generated files
- snapshots
- formatting-only edits
- comment-only changes
- bulk renames or moves with no behavior change
- boilerplate repeated consistently

For dependency upgrades, do **not** treat lockfile size as review size. A version bump plus lockfile regeneration with little or no code adaptation is usually `size/xs` or `size/s`. Escalate only when the upgrade is major, requires code/config/test changes, or changes runtime behavior.

Classify by reviewer effort:
- `size/xs`: trivial docs/tests/formatting or routine dependency bump
- `size/s`: localized change in one area, cheap review
- `size/m`: several meaningful files or one contained feature slice
- `size/l`: cross-module, risky, or contract-changing work
- `size/xl`: broad cross-cutting or split-worthy PR

When the PR is part of a stack, judge only the incremental diff against its base branch.

When you comment on PR size, always include:
1. the chosen label
2. 2-3 reasons
3. any discounted churn
4. any escalators that increased the size
