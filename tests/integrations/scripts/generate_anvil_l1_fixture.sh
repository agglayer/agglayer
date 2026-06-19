#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../../.." && pwd)
FIXTURE_DIR=${1:-"$REPO_ROOT/tests/integrations/fixtures/anvil-l1"}

DOCKER_IMAGE=${DOCKER_IMAGE:-hermeznetwork/geth-zkevm-contracts}
ANVIL_HOST=${ANVIL_HOST:-127.0.0.1}
ANVIL_PORT=${ANVIL_PORT:-18545}

require() {
    command -v "$1" >/dev/null 2>&1 || {
        printf 'missing required command: %s\n' "$1" >&2
        exit 1
    }
}

for tool in anvil cast docker jq mktemp tr; do
    require "$tool"
done

mkdir -p "$FIXTURE_DIR"

workdir=$(mktemp -d)

cleanup() {
    if [ -n "${replay_pid:-}" ]; then
        kill "$replay_pid" >/dev/null 2>&1 || true
    fi

    if [ -n "${container_id:-}" ]; then
        docker rm -f "$container_id" >/dev/null 2>&1 || true
    fi

    rm -rf "$workdir"
}

trap cleanup EXIT

container_id=$(docker run -d -p 127.0.0.1::8545 "$DOCKER_IMAGE")
docker cp "$container_id":/config/genesis.json "$workdir/genesis.json"
docker cp "$container_id":/config/deploy_output.json "$workdir/deploy_output.json"

docker_binding=$(docker port "$container_id" 8545/tcp)
docker_port=${docker_binding##*:}
docker_rpc="http://127.0.0.1:${docker_port}"

for _ in $(seq 1 60); do
    if cast block-number --rpc-url "$docker_rpc" >/dev/null 2>&1; then
        break
    fi

    sleep 1
done

cast block-number --rpc-url "$docker_rpc" >/dev/null 2>&1 || {
    printf 'Docker RPC never became ready at %s\n' "$docker_rpc" >&2
    exit 1
}

replay_rpc="http://${ANVIL_HOST}:${ANVIL_PORT}"

anvil \
    --init "$workdir/genesis.json" \
    --host "$ANVIL_HOST" \
    --port "$ANVIL_PORT" \
    --chain-id 1337 \
    --no-mining \
    --order fifo \
    --auto-impersonate \
    >"$workdir/anvil.log" 2>&1 &
replay_pid=$!

for _ in $(seq 1 60); do
    if cast block-number --rpc-url "$replay_rpc" >/dev/null 2>&1; then
        break
    fi

    sleep 1
done

cast block-number --rpc-url "$replay_rpc" >/dev/null 2>&1 || {
    printf 'Anvil replay RPC never became ready at %s\n' "$replay_rpc" >&2
    exit 1
}

# The shipped genesis is only the deployment base. Patch in the live Docker
# block-0 EOAs before replaying blocks so the later transactions reproduce
# exactly.
#
# Address groups below:
# - the first 10 addresses are the standard Foundry mnemonic accounts
# - 0xf39f... is also the Docker deployer/admin account
# - 0x7099... is also the trusted aggregator account
# - 0x4acf... and 0x5c68... are extra bootstrap EOAs funded and used by the
#   Docker contracts chain during its initial setup
for address in \
    0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 \
    0x70997970c51812dc3a010c7d01b50e0d17dc79c8 \
    0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc \
    0x90f79bf6eb2c4f870365e785982e1f101e93b906 \
    0x15d34aaf54267db7d7c367839aaf71a00a2c6a65 \
    0x9965507d1a55bcc2695c58ba16fb37d819b0a4dc \
    0x976ea74026e726554db657fa54763abd0c3a0aa9 \
    0x14dc79964da2c08b23698b3d3cc7ca32193d9955 \
    0x23618e81e3f5cdf7f54c3d65f7fbc0abf5b21e8f \
    0xa0ee7a142d267c1f36714e4a8f75612f20a79720 \
    0x4acfcbe27910bc7428155c571540fd844ec0cf10 \
    0x5c68bc17089af0b422b682da023a7abae120e4ed
do
    balance=$(cast rpc --rpc-url "$docker_rpc" eth_getBalance "$address" 0x0 | tr -d '"')
    nonce=$(cast rpc --rpc-url "$docker_rpc" eth_getTransactionCount "$address" 0x0 | tr -d '"')

    cast rpc --rpc-url "$replay_rpc" anvil_setBalance "$address" "$balance" >/dev/null
    cast rpc --rpc-url "$replay_rpc" anvil_setNonce "$address" "$nonce" >/dev/null
done

latest_block=$(cast block-number --rpc-url "$docker_rpc")

for block_number in $(seq 1 "$latest_block"); do
    block_json=$(cast rpc --rpc-url "$docker_rpc" eth_getBlockByNumber "0x$(printf '%x' "$block_number")" true)
    block_timestamp_hex=$(printf '%s' "$block_json" | jq -r '.timestamp')
    block_timestamp_dec=$((block_timestamp_hex))

    cast rpc --rpc-url "$replay_rpc" evm_setNextBlockTimestamp "$block_timestamp_dec" >/dev/null

    tx_count=$(printf '%s' "$block_json" | jq '.transactions | length')
    if [ "$tx_count" -gt 0 ]; then
        while IFS= read -r tx_json; do
            tx_request=$(printf '%s' "$tx_json" | jq -c '
                if .type == "0x2" then
                    {
                        from,
                        nonce,
                        gas,
                        value,
                        input,
                        to,
                        maxFeePerGas,
                        maxPriorityFeePerGas,
                        accessList
                    }
                else
                    {
                        from,
                        nonce,
                        gas,
                        value,
                        input,
                        to,
                        gasPrice
                    }
                end
                | with_entries(select(.value != null))
            ')

            cast rpc --rpc-url "$replay_rpc" eth_sendTransaction "$tx_request" >/dev/null
        done <<EOF
$(printf '%s' "$block_json" | jq -c '.transactions[]')
EOF
    fi

    cast rpc --rpc-url "$replay_rpc" anvil_mine 1 >/dev/null
done

state_blob=$(cast rpc --rpc-url "$replay_rpc" anvil_dumpState | tr -d '"')
printf '%s' "$state_blob" > "$FIXTURE_DIR/state.hex"

manager_code=$(cast code 0x0B306BF915C4d645ff596e518fAf3F9669b97016 --rpc-url "$replay_rpc")
init_logs=$(cast rpc --rpc-url "$replay_rpc" eth_getLogs '{"fromBlock":"0x0","toBlock":"latest","address":"0x610178dA211FEF7D417bC0e6FeD39F05609AD788"}')
default_l1_info_root=$(cast call 0x610178dA211FEF7D417bC0e6FeD39F05609AD788 'l1InfoRootMap(uint32)(bytes32)' 0 --rpc-url "$replay_rpc")

if [ "$manager_code" = "0x" ]; then
    printf 'replayed chain is missing PolygonRollupManager code\n' >&2
    exit 1
fi

if [ "$init_logs" = '[]' ]; then
    printf 'replayed chain is missing GlobalExitRoot history\n' >&2
    exit 1
fi

# Record the anvil/foundry version that produced this fixture. anvil_dumpState's
# serialized format is coupled to the foundry version, so the loader (and CI, via
# foundry-toolchain in .github/workflows/test.yml) must use a matching anvil.
# Extract the semver token from the first line rather than stripping to the last
# whitespace-separated word: verbose build strings (e.g.
# "anvil 1.7.1 (sha 2024-08-07T07:23:08Z)") would otherwise capture the trailing
# timestamp instead of the version.
anvil_version=$(anvil --version | head -n1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.]+)?' | head -n1)

jq -n \
    --arg source_image "$DOCKER_IMAGE" \
    --arg foundry_version "$anvil_version" \
    --arg latest_block "$latest_block" \
    --arg chain_id '1337' \
    --arg default_l1_info_root "$default_l1_info_root" \
    --arg generated_with 'tests/integrations/scripts/generate_anvil_l1_fixture.sh' \
    --slurpfile deploy_output "$workdir/deploy_output.json" \
    '{
        source_image: $source_image,
        foundry_version: $foundry_version,
        docker_latest_block: ($latest_block | tonumber),
        chain_id: ($chain_id | tonumber),
        default_l1_info_root: $default_l1_info_root,
        generated_with: $generated_with,
        deploy_output: $deploy_output[0]
    }' > "$FIXTURE_DIR/metadata.json"

printf 'wrote %s/state.hex\n' "$FIXTURE_DIR"
