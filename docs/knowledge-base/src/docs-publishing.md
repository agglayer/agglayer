# Documentation publishing

Documentation is built and published by `.github/workflows/doc.yml`
on every PR and every push to `main`.

The pipeline publishes two outputs together:

- **Knowledge base (mdbook):** `docs/knowledge-base/`.
- **Rust API docs (rustdoc):** `cargo doc --no-deps --all --all-features`.

The deployed site uses this layout:

- `/` -> knowledge-base landing page.
- `/rustdoc/agglayer/` -> Rust API docs.

| Trigger | URL |
|---|---|
| Push to `main` | GitHub Pages (`https://agglayer.github.io/agglayer/`) |
| Pull request | Cloudflare Workers preview (`https://<repo>-pr-<PR_NUMBER>-rust-docs.agglayer.dev`) |

For PR previews,
the workflow posts the deployed URL as a PR comment automatically.

## Merge-queue behavior

The project uses GitHub merge queue.
Two events matter:

- `merge_group`: pre-merge validation builds docs but does not deploy.
- `push` to `main`: deployment to GitHub Pages after merge queue completion.

The `deploy-gh-pages` job guards against fork deployment
with `!github.event.repository.fork`.

## Contributor expectations

- For knowledge-base changes,
  ensure `mdbook build docs/knowledge-base/` succeeds locally.
- For API changes,
  keep `///` Rust docs accurate and complete for public and `pub(crate)` items.
- When linking between rustdoc items,
  avoid links known to break for `pub(crate)` cross-item references.

Agglayer crates are not published to crates.io,
so docs.rs is not the canonical documentation surface.
GitHub Pages is the canonical published location.
