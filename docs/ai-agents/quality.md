# Definition of done

- Documentation-only changes are not exempt: run verification before declaring completion.
- Default verification: run `cargo check --workspace --tests --all-features`.
- For code behavior changes, run `cargo make ci-all`.
- Report exact command(s) and whether each passed or failed.
- If checks fail, attempt focused fixes for failures plausibly caused by your changes, then rerun checks.
- Do not loop: stop after 2 fix-and-rerun cycles or if failures appear unrelated to your changes, then hand control back with a brief summary.
