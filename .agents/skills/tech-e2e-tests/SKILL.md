---
name: tech-e2e-tests
description: >
  Architecture and agent rules for kurtosis-cdk E2E tests.
  Load when working on or debugging E2E tests, CI failures in the e2e job,
  or any task involving bridge testing or kurtosis.
---

# E2E test architecture

## How CI runs E2E

The CI workflow `.github/workflows/test.yml`
(job `call-agglayer-node-e2e-workflow`) calls an external reusable workflow at
`agglayer/e2e/.github/workflows/agglayer-node-e2e.yml`.
It passes:

- A freshly built Docker image (via `docker-image-override: agglayer_image`)
- A pinned `kurtosis-cdk-ref` commit
- A pinned `agglayer-e2e-ref` commit
- A `kurtosis-cdk-args` JSON block configuring the network
  (including `l1_el_type`, `sequencer_type`, and `reth_image`
  for the reth-based L1)

The workflow runs
`bats tests/agglayer/bridges.bats tests/agglayer/rpc-tests.bats --filter-tags agglayer`
in the `agglayer/e2e` repo.
The `bridges.bats` file covers L1/L2 bridging;
`rpc-tests.bats` validates reth-specific RPC methods
(e.g. `eth_getTransactionBySenderAndNonce`).

E2E jobs only trigger on `merge_group` and `workflow_dispatch` events,
not on `pull_request`.
See [Validating E2E on a PR branch](#validating-e2e-on-a-pr-branch)
for how to exercise them before merging.

## Rules

- The `agglayer/e2e` repo bundles its own bats-support/bats-assert libraries
  in `core/helpers/lib/`.
- The tests use `polycli ulxly bridge asset` and `polycli ulxly claim asset`
  for bridge operations.
- The test environment is bootstrapped by sourcing `tests/.env` and
  `core/helpers/common.bash` (`_setup_vars`), which auto-discovers RPC URLs,
  bridge addresses, and private keys from the running kurtosis enclave.
- All pinned commits and args live in `.github/workflows/test.yml`;
  that is the single source of truth.
- **Always ask whether to rebuild `agglayer:local` before running.**
  Recommend rebuilding (`docker build -t agglayer:local .` from the repo root)
  because skipping means the tests run against a stale image
  that does not reflect the current code.
- **polycli version must match CI.**
  The reusable workflow `agglayer/e2e/.github/workflows/agglayer-node-e2e.yml`
  pins `POLYCLI_VERSION` (e.g. `v0.1.90`) as an env var near the top.
  Install the matching release binary rather than building from source,
  because `make install` produces the latest dev build which may have
  regressions (e.g. gas estimation changes that cause bridge tx reverts).
  To install a specific version:
  ```
  curl -sL "https://github.com/0xPolygon/polygon-cli/releases/download/${POLYCLI_VERSION}/polycli_${POLYCLI_VERSION}_linux_amd64.tar.gz" \
    | tar xz -C ~/go/bin/ && mv ~/go/bin/polycli_* ~/go/bin/polycli
  ```
- `polycli` lives at `~/go/bin/polycli` and is not on the default PATH.
  Export `PATH="$PATH:$HOME/go/bin"` before running bats or any polycli commands.
- External repos are cached under `/tmp/agglayer-e2e/`.
  Before cloning, check whether they already exist at the correct refs
  (compare `git -C <dir> rev-parse HEAD` with the pinned SHA in
  `.github/workflows/test.yml`).
  Re-clone only if the ref has changed.
- The kurtosis args file at `/tmp/agglayer-e2e/kurtosis-args.json`
  should be checked/regenerated from `test.yml` each run
  (ensure `"agglayer_image": "agglayer:local"` is present in `args`).

- **When e2e tests fail locally but pass in CI, check tool versions first.**
  Compare local versions of polycli, kurtosis, bats, and foundry/cast
  against the versions CI installs (defined in `agglayer-node-e2e.yml`).
  Version drift is a common root cause of otherwise-mysterious failures
  (e.g. gas estimation regressions, changed CLI flags).

For step-by-step local setup, see `docs/knowledge-base/e2e-tests.md`.

## Validating E2E on a PR branch

E2E jobs only run on `merge_group` and `workflow_dispatch` events,
so they are skipped during normal PR CI.
To exercise them before the PR enters the merge queue,
trigger a `workflow_dispatch` run against the PR branch:

```bash
gh workflow run test.yml --repo agglayer/agglayer --ref <branch-name>
```

Monitor the run with:

```bash
gh run list --repo agglayer/agglayer --branch <branch-name> --workflow test.yml --limit 1
gh run view <run-id> --repo agglayer/agglayer --json jobs \
  --jq '.jobs[] | "\(.name)\t\(.conclusion)"'
```

This run includes the full E2E suite (docker build + kurtosis + bats tests)
and typically takes ~30 minutes.
