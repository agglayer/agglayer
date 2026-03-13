# Running kurtosis-cdk E2E tests locally

This document describes how to run the CI-style end-to-end tests locally.
These tests spin up a full CDK network (L1 + L2 + bridge + agglayer) using
[kurtosis-cdk](https://github.com/0xPolygon/kurtosis-cdk),
inject a locally built `agglayer` Docker image,
then run BATS bridge tests from the
[agglayer/e2e](https://github.com/agglayer/e2e) repository.

In CI, these tests run on `merge_group` and `workflow_dispatch` events only
(see `.github/workflows/test.yml`, job `call-cdk-e2e-workflow`).

## Prerequisites

| Tool | Install (Ubuntu) | Notes |
|---|---|---|
| Docker (with BuildKit/buildx) | `sudo apt install docker.io docker-buildx` | Engine must be running |
| [Kurtosis CLI](https://docs.kurtosis.com/install/) | `sudo apt install kurtosis-cli` (after adding the apt repo) | See install docs for repo setup |
| [bats](https://github.com/bats-core/bats-core) | `sudo apt install bats` | Test runner (bats-support/bats-assert are bundled in the e2e repo) |
| [Foundry (cast)](https://getfoundry.sh/) | `curl -L https://foundry.paradigm.xyz \| bash && foundryup` | On-chain interactions |
| [polycli](https://github.com/0xPolygon/polygon-cli) | See [Installing polycli](#installing-polycli) below | Bridge test helper; **version must match CI** |
| jq, bc, curl | `sudo apt install jq bc curl` | Usually pre-installed |

Make sure `~/go/bin` is on your `$PATH`.

### Installing polycli

The CI workflow (`agglayer/e2e/.github/workflows/cdk-e2e.yml`) pins a specific
polycli version in its `POLYCLI_VERSION` env var. **You must use the same
version locally.** Building from source with `make install` produces the latest
dev build, which may have regressions (e.g. gas estimation changes that cause
bridge transactions to revert with `OutOfGas`).

To install the CI-pinned version:

```bash
# Check the pinned version in agglayer/e2e's cdk-e2e.yml (e.g. v0.1.90)
POLYCLI_VERSION=v0.1.90

curl -sL "https://github.com/0xPolygon/polygon-cli/releases/download/${POLYCLI_VERSION}/polycli_${POLYCLI_VERSION}_linux_amd64.tar.gz" \
  | tar xz -C /tmp/
mv /tmp/polycli_* ~/go/bin/polycli

polycli version  # should print the expected version
```

## Step-by-step

### 1. Build the agglayer Docker image

Rebuild before running tests so the image reflects the current source tree.
Skipping this step means the tests run against a stale image.

From the repository root:

```bash
docker build -t agglayer:local .
```

This takes 10-20 minutes on a first build (Rust compilation + SP1 circuit download).
Subsequent builds are faster thanks to Docker layer caching.

### 2. Clone the external repositories

The CI workflow in `.github/workflows/test.yml` (job `call-cdk-e2e-workflow`)
pins specific commits for `kurtosis-cdk-ref` and `agglayer-e2e-ref`.
Look there for the current values and substitute them below:

```bash
# Look up current values in .github/workflows/test.yml
KURTOSIS_CDK_REF=<kurtosis-cdk-ref from test.yml>
AGGLAYER_E2E_REF=<agglayer-e2e-ref from test.yml>

mkdir -p /tmp/agglayer-e2e

git clone https://github.com/0xPolygon/kurtosis-cdk.git /tmp/agglayer-e2e/kurtosis-cdk
cd /tmp/agglayer-e2e/kurtosis-cdk
git fetch origin "$KURTOSIS_CDK_REF" && git checkout "$KURTOSIS_CDK_REF"

git clone https://github.com/agglayer/e2e.git /tmp/agglayer-e2e/agglayer-e2e
cd /tmp/agglayer-e2e/agglayer-e2e
git fetch origin "$AGGLAYER_E2E_REF" && git checkout "$AGGLAYER_E2E_REF"
```

### 3. Create the kurtosis args file

Copy the `kurtosis-cdk-args` JSON from the same CI job in `test.yml`
and add the local Docker image override (`"agglayer_image": "agglayer:local"`):

```bash
cat > /tmp/agglayer-e2e/kurtosis-args.json << 'EOF'
<kurtosis-cdk-args from test.yml, with "agglayer_image": "agglayer:local" added to "args">
EOF
```

### 4. Start the kurtosis network

```bash
kurtosis engine start
kurtosis clean --all
kurtosis run \
  --enclave cdk \
  --args-file /tmp/agglayer-e2e/kurtosis-args.json \
  /tmp/agglayer-e2e/kurtosis-cdk
```

This pulls ~20 container images and deploys contracts on L1.
Takes 5-15 minutes depending on network speed.
Verify with:

```bash
kurtosis enclave inspect cdk
```

All services should show `RUNNING`, including `agglayer`.

### 5. Run the BATS tests

```bash
cd /tmp/agglayer-e2e/agglayer-e2e

# Load the test environment
set -a
source ./tests/.env
set +a

export BATS_LIB_PATH="$PWD/core/helpers/lib"
export PROJECT_ROOT="$PWD"
export ENCLAVE_NAME="cdk"
export PATH="$PATH:$HOME/go/bin"   # needed if polycli was installed via 'make install'

# Run the bridge tests (same as CI for test-name "agglayer-bridging")
bats tests/lxly/lxly.bats
```

The tests exercise L1-to-L2 and L2-to-L1 native ETH bridging,
plus L2-originated ERC20 token bridging round-trips.

### 6. Clean up

```bash
kurtosis enclave stop cdk
kurtosis clean --all
```

## Debugging failures

- **Check tool versions first.** If tests fail locally but pass in CI,
  compare your local versions of polycli, kurtosis, bats, and foundry/cast
  against the versions installed by `cdk-e2e.yml`. Version drift is a common
  root cause -- for example, a newer polycli may change gas estimation behavior,
  causing bridge transactions to revert with `OutOfGas`.
- **Dump enclave logs**: `kurtosis dump ./dump` saves all container logs.
- **Inspect a service**: `kurtosis service logs cdk agglayer` for agglayer logs.
- **Shell into a container**: `kurtosis service shell cdk agglayer`.
- **Check ports**: `kurtosis port print cdk <service> <port-name>` to get the
  mapped local URL for any service.
- **Trace a failed transaction**: use `cast run <txHash> --rpc-url <rpc>`
  to get a full execution trace including the exact revert reason.
