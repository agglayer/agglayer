# Rust API documentation publishing

`cargo doc` is built and published by `.github/workflows/doc.yml` on every PR
and main-branch push.

| Trigger | URL |
|---|---|
| Push to `main` | GitHub Pages (`https://agglayer.github.io/agglayer/`) |
| Pull request | Cloudflare Workers — URL is posted as a PR comment automatically: `https://agglayer-pr-<PR_NUMBER>-rust-docs.agglayer.dev` |

The crates are **not** published to crates.io, so docs.rs has no content for
them.

## Implication for doc comments

`///` Rust doc comments are the primary public documentation surface for this
project. Add them to all public (and `pub(crate)`) items that callers need to
understand. Changes merged to `main` are reflected at the GitHub Pages URL
above.

Note: intra-doc links to `pub(crate)` items from other `pub(crate)` items in
the same crate will generate a broken-link warning at `rustdoc` time. Use plain
text or a code span instead of a bracketed link in those cases.
