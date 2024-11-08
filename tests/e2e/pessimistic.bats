setup() {
    load 'helpers/common-setup'
    _common_setup
    load 'helpers/common'
    load 'helpers/lxly-bridge-test'

    if [ -z "$BRIDGE_ADDRESS" ]; then
        local combined_json_file="/opt/zkevm/combined.json"
        echo "BRIDGE_ADDRESS env variable is not provided, resolving the bridge address from the Kurtosis CDK '$combined_json_file'" >&3

        # Fetching the combined JSON output and filtering to get polygonZkEVMBridgeAddress
        combined_json_output=$($contracts_service_wrapper "cat $combined_json_file" | tail -n +2)
        bridge_default_address=$(echo "$combined_json_output" | jq -r .polygonZkEVMBridgeAddress)
        BRIDGE_ADDRESS=$bridge_default_address
    fi
    echo "Bridge address=$BRIDGE_ADDRESS" >&3

    readonly sender_private_key=${SENDER_PRIVATE_KEY:-"12d7de8621a77640c9241b2595ba78ce443d05e94090365ab3bb5e19df82c625"}
    readonly sender_addr="$(cast wallet address --private-key $sender_private_key)"
    destination_net=${DESTINATION_NET:-"1"}
    destination_addr=${DESTINATION_ADDRESS:-"0x0bb7AA0b4FdC2D2862c088424260e99ed6299148"}
    ether_value=${ETHER_VALUE:-"0.0200000054"}
    amount=$(cast to-wei $ether_value ether)
    readonly native_token_addr=${NATIVE_TOKEN_ADDRESS:-"0x0000000000000000000000000000000000000000"}

    # no gas_token_addr

    readonly is_forced=${IS_FORCED:-"true"}
    readonly bridge_addr=$BRIDGE_ADDRESS
    readonly meta_bytes=${META_BYTES:-"0x"}

    readonly l1_rpc_url=${L1_ETH_RPC_URL:-"$(kurtosis port print $enclave el-1-geth-lighthouse rpc)"}
    readonly bridge_api_url=${BRIDGE_API_URL:-"$(kurtosis port print $enclave zkevm-bridge-service-001 rpc)"}

    readonly dry_run=${DRY_RUN:-"false"}
    readonly l1_rpc_network_id=$(cast call --rpc-url $l1_rpc_url $bridge_addr 'networkID() (uint32)')
    readonly l2_rpc_network_id=$(cast call --rpc-url $l2_rpc_url $bridge_addr 'networkID() (uint32)')
    gas_price=$(cast gas-price --rpc-url "$l2_rpc_url")
    readonly weth_token_addr=$(cast call --rpc-url $l2_rpc_url $bridge_addr 'WETHToken()' | cast parse-bytes32-address)
}

@test "transfer L1 to L2 to L1" {

    destination_addr=$sender_addr
    destination_net=1
    echo "*** bridgeAsset L1 -> L2 dest: $destination_addr eth:$ether_value" >&3
    run bridgeAsset "0x0000000000000000000000000000000000000000" "$l1_rpc_url"
    assert_success

    echo "Claim in L2" >&3
    timeout="120"
    claim_frequency="10"
    run wait_for_claim "$timeout" "$claim_frequency" "$l2_rpc_url"
    assert_success

    echo "*** bridgeAsset L2 -> L1" >&3
    destination_addr=$sender_addr
    destination_net=0
    run bridgeAsset "0x0000000000000000000000000000000000000000" "$l2_rpc_url"
    assert_success

    echo "*** Claim in L1" >&3
    timeout="180"
    claim_frequency="10"
    run wait_for_claim "$timeout" "$claim_frequency" "$l1_rpc_url"
    assert_success
}
