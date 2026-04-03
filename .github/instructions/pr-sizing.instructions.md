---
applyTo: "**"
---

When reviewing a pull request, classify its size by **review effort and semantic complexity**, not raw line count.

Use the GitHub Project `Size` field values: `XS`, `S`, `M`, `L`, `XL`.

Discount mechanical churn when it does not add real review cost:
- lockfiles
- generated files
- snapshots
- formatting-only edits
- comment-only changes
- bulk renames or moves with no behavior change
- boilerplate repeated consistently

For dependency upgrades, do **not** treat lockfile size as review size. A version bump plus lockfile regeneration with little or no code adaptation is usually `XS` or `S`. Escalate only when the upgrade is major, requires code/config/test changes, or changes runtime behavior.

Classify by reviewer effort:
- `XS`: trivial docs/tests/formatting or routine dependency bump
- `S`: localized change in one area, cheap review
- `M`: several meaningful files or one contained feature slice
- `L`: cross-module, risky, or contract-changing work
- `XL`: broad cross-cutting or split-worthy PR

When the PR is part of a stack, judge only the incremental diff against its base branch.

When you comment on PR size, always include:
1. the chosen size
2. 2-3 reasons
3. any discounted churn
4. any escalators that increased the size
