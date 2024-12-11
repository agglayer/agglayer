setup() {
    PROJECT_ROOT="$(cd "$(dirname "$BATS_TEST_FILENAME")/.." >/dev/null 2>&1 && pwd)"

    load 'helpers/kurtosis'
    load 'helpers/cdk'
    load 'helpers/common'
    setup_kurtosis "$KURTOSIS_PATH"

    readonly enclave="pp"
    readonly max_verification_batch_retry=10

    readonly tmp_dir=$PROJECT_ROOT/tmp/$BATS_TEST_NAME
    mkdir -p $tmp_dir >&3
    echo "Temporary directory: $tmp_dir" >&3
    # start_network $enclave $kurtosis_path/.github/tests/fork12-pessimistic.yml
    # start_network $enclave $kurtosis_path/.github/tests/attach-second-cdk.yml

    # Grab the combined.json files for future ref
    $kurtosis service exec $enclave contracts-001 "cat /opt/zkevm/combined-001.json"  | tail -n +2 | jq '.' > $tmp_dir/combined-001.json
    $kurtosis service exec $enclave contracts-002 "cat /opt/zkevm/combined-002.json"  | tail -n +2 | jq '.' > $tmp_dir/combined-002.json

    readonly l1_rpc_url=$($kurtosis port print $enclave el-1-geth-lighthouse rpc)

    # Let's confirm the real verifier was deployed for the first rollup. At this point, I'm just going
    run cast code --rpc-url $l1_rpc_url $(cat $tmp_dir/combined-001.json | jq -r '.verifierAddress')
    assert_success
    refute_output "0x"

    run cast code --rpc-url $l1_rpc_url $(cat $tmp_dir/combined-002.json | jq -r '.verifierAddress')
    assert_success
    refute_output "0x"

    # Check that the hash of the verifier is actually the sp1 verifier. It should be f33fc6bc90b5ea5e0272a7ab87d701bdd05ecd78b8111ca4f450eeff1e6df26a
    deployed_bytecode=$($kurtosis service exec $enclave contracts-001 'cat /opt/zkevm-contracts/artifacts/contracts/verifiers/SP1Verifier.sol/SP1Verifier.json' \
        | tail -n +2 | jq -r '.deployedBytecode' | sha256sum)

    run bash -c "cast code --rpc-url $l1_rpc_url $(cat $tmp_dir/combined-001.json | jq -r '.verifierAddress') | sha256sum"
    assert_success
    assert_output "$deployed_bytecode"
}

@test "Bali::2024-11-25: multi network pp regression test" {
    # At this point, the agglayer config needs to be manually updated for rollup2.
    # This will add a second entry to the agglayer config
    # TODO in the future we might as well make this values by default...
    run $kurtosis service exec $enclave agglayer "sed -i 's/\[proof\-signers\]/2 = \"http:\/\/cdk-erigon-rpc-002:8123\"\n\[proof-signers\]/i' /etc/zkevm/agglayer-config.toml"
    echo "Agglayer config updated" >&3

    run $kurtosis service stop $enclave agglayer
    # assert_success
    echo "Agglayer stopped" >&3
    # #
    run $kurtosis service start $enclave agglayer
    # assert_success
    echo "Waiting for the agglayer to come back up..." >&3

    local n1_rpc_url=$($kurtosis port print $enclave cdk-erigon-rpc-001 rpc)
    local n2_rpc_url=$($kurtosis port print $enclave cdk-erigon-rpc-002 rpc)

    echo "Checking if BatchNumber is increasing on both chains" >&3

    # Take a lot of time
    # cdk_check_virtual_batch_increase $n1_rpc_url $max_verification_batch_retry
    # cdk_check_verified_batch_increase $n1_rpc_url $max_verification_batch_retry

    run cast wallet new
    assert_success

    target_address=$(echo $output | sed -n 's/.*Address:[^0]*\([^\ ]*\).*/\1/p')
    target_private_key=$(echo $output | sed -n 's/.*Private key:[^0]*\([^\ ]*\).*/\1/p')

    # Let's setup some variables for future use
    private_key="0x12d7de8621a77640c9241b2595ba78ce443d05e94090365ab3bb5e19df82c625"
    eth_address=$(cast wallet address --private-key $private_key)
    l2_pp_url=$($kurtosis port print $enclave cdk-erigon-rpc-001 rpc)
    l2_fep_url=$($kurtosis port print $enclave cdk-erigon-rpc-002 rpc)
    bridge_address=$(cat $tmp_dir/combined-001.json | jq -r .polygonZkEVMBridgeAddress)
    pol_address=$(cat $tmp_dir/combined-001.json | jq -r .polTokenAddress)

    # Now let's make sure we have balance everywhere
    cast balance --ether --rpc-url $l1_rpc_url $eth_address
    cast balance --ether --rpc-url $l2_pp_url $eth_address
    cast balance --ether --rpc-url $l2_fep_url $eth_address

    echo "Let's fund the claim tx manager for both rollups. These address come from the chain configurations (so either input_parser or the args file)" >&3
    cast send --legacy --value 10ether --rpc-url $l2_pp_url --private-key $private_key 0x5f5dB0D4D58310F53713eF4Df80ba6717868A9f8
    cast send --legacy --value 10ether --rpc-url $l2_fep_url --private-key $private_key 0x93F63c24735f45Cd0266E87353071B64dd86bc05

    echo "Let's mint some POL for bridge testing" >&3
    cast send \
         --rpc-url $l1_rpc_url \
         --private-key $private_key \
         $pol_address \
         'mint(address,uint256)' \
         $eth_address 10000000000000000000000

    echo "We also need to approve" >&3
    cast send \
         --rpc-url $l1_rpc_url \
         --private-key $private_key \
         $pol_address \
         'approve(address,uint256)' \
         $bridge_address 10000000000000000000000


    # Let's go to madison county

    echo "R0, R1, Native, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 100000000000000000000 \
        --private-key $private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        1 $target_address 100000000000000000000 $(cast az) true 0x

    echo "R1, R2, Native, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 100000000000000000000 \
        --private-key $private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        2 $target_address 100000000000000000000 $(cast az) true 0x

    echo "R0, R1, Native, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 1000000000000000000 \
        --private-key $private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        1 $target_address 1000000000000000000 $(cast az) false 0x

    echo "R0, R2, Native, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 1000000000000000000 \
        --private-key $private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        2 $target_address 1000000000000000000 $(cast az) false 0x

    echo "R0, R1, POL, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --private-key $private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        1 $target_address 1000000000000000000 $pol_address true 0x

    echo "R0, R2, POL, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --private-key $private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        2 $target_address 1000000000000000000 $pol_address true 0x

    sleep 30

    echo "Let's see if our balances have grown (both should be non-zero)" >&3
    run check_balance $l2_pp_url $target_address
    assert_success
    echo "Let's see if our balances have grown (both should be non-zero)" >&3
    run check_balance $l2_fep_url $target_address
    assert_success

    echo "Let's check our L2 token balance (both should be non-zero)" >&3
    token_hash=$(cast keccak $(cast abi-encode --packed 'f(uint32,address)' 0 $pol_address))
    l2_pol_address=$(cast call --rpc-url $l2_pp_url  $bridge_address 'tokenInfoToWrappedToken(bytes32)(address)' $token_hash)
    # TODO: Fix the parsing of those balance
    # run check_balance_rpc_call $l2_pp_url $l2_pol_address 'balanceOf(address)(uint256)' $target_address
    # assert_success
    # cast call --rpc-url $l2_fep_url $l2_pol_address 'balanceOf(address)(uint256)' $target_address

    sleep 30

    echo "We should be in a good position now to try some bridge exits!!" >&3
    echo "PP Exits" >&3
    echo "R1, R0, Native, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_pp_url \
        --value 100000000000000003 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        0 $target_address 100000000000000003 $(cast az) false 0x

    echo "R1, R2, Native, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_pp_url \
        --value 100000000000000004 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        2 $target_address 100000000000000004 $(cast az) false 0x

    echo "R1, R0, POL, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_pp_url \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        0 $target_address 100000000000000005 $l2_pol_address false 0x

    echo "R1, R2, POL, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_pp_url \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        2 $target_address 100000000000000006 $l2_pol_address false 0x

    echo "R1, R0, Native, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_pp_url \
        --value 100000000000000001 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        0 $target_address 100000000000000001 $(cast az) true 0x

    echo "R1, R2, Native, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_pp_url \
        --value 100000000000000002 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        2 $target_address 100000000000000002 $(cast az) true 0x

    echo "FEP Exists" >&3
    echo "R2, R0, Native, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_fep_url \
        --value 100000000000000007 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        0 $target_address 100000000000000007 $(cast az) false 0x

    echo "R2, R1, Native, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_fep_url \
        --value 100000000000000008 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        1 $target_address 100000000000000008 $(cast az) false 0x

    echo "R2, R0, POL, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_fep_url \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        0 $target_address 100000000000000009 $l2_pol_address false 0x

    echo "R2, R1, POL, No Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_fep_url \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        1 $target_address 100000000000000010 $l2_pol_address false 0x

    echo "R2, R0, Native, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_fep_url \
        --value 100000000000000011 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        0 $target_address 100000000000000011 $(cast az) true 0x

    echo "R2, R1, Native, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l2_fep_url \
        --value 100000000000000012 \
        --private-key $target_private_key \
        $bridge_address \
        "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
        1 $target_address 100000000000000012 $(cast az) true 0x

    echo "Do some criss-crossing" >&3
    for i in {1..20}; do
        echo "R1, R0, Native, Force" >&3
        cast send \
             --legacy \
             --rpc-url $l2_pp_url \
             --value 100000000000000001 \
             --private-key $target_private_key \
             $bridge_address \
             "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
             0 $target_address 100000000000000001 $(cast az) true 0x

        echo "R2, R0, Native, Force" >&3
        cast send \
             --legacy \
             --rpc-url $l2_fep_url \
             --value 100000000000000011 \
             --private-key $target_private_key \
             $bridge_address \
             "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
             0 $target_address 100000000000000011 $(cast az) true 0x
    done


    echo "Do a bridge message" >&3
    echo "R0, R1, Message, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 1 \
        --private-key $private_key \
        $bridge_address \
        "bridgeMessage(uint32,address,bool,bytes)" \
        1 "0xFFFF000000000000000000000000000000000001" "true" "0x1234"

    echo "R0, R2, Message, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 1 \
        --private-key $private_key \
        $bridge_address \
        "bridgeMessage(uint32,address,bool,bytes)" \
        2 "0xFFFF000000000000000000000000000000000001" "true" "0x1234"

    echo "R0, R9999, Message, Force" >&3
    cast send \
        --legacy \
        --rpc-url $l1_rpc_url \
        --value 1 \
        --private-key $private_key \
        $bridge_address \
        "bridgeMessage(uint32,address,bool,bytes)" \
        9999 "0xFFFF000000000000000000000000000000000001" "true" "0x1234"

    for i in {2..16}; do
        call_size=$(bc <<< "2^$i")
        echo "R0, R1, Message, Force" >&3
        cast send \
             --legacy \
             --rpc-url $l1_rpc_url \
             --value 1 \
             --private-key $private_key \
             $bridge_address \
             "bridgeMessage(uint32,address,bool,bytes)" \
             1 "0xFFFF000000000000000000000000000000000001" "true" "0x$(cat /dev/random | xxd -p | tr -d "\n" | head -c $call_size)"

        echo "R0, R2, Message, Force" >&3
        cast send \
             --legacy \
             --rpc-url $l1_rpc_url \
             --value 1 \
             --private-key $private_key \
             $bridge_address \
             "bridgeMessage(uint32,address,bool,bytes)" \
             2 "0xFFFF000000000000000000000000000000000001" "true" "0x$(cat /dev/random | xxd -p | tr -d "\n" | head -c $call_size)"
    done


    for i in {2..16}; do
        call_size=$(bc <<< "2^$i")
        echo "R1, R0, Message, Force" >&3
        cast send \
             --legacy \
             --rpc-url $l2_pp_url \
             --value 1 \
             --private-key $target_private_key \
             $bridge_address \
             "bridgeMessage(uint32,address,bool,bytes)" \
             0 "0xFFFF000000000000000000000000000000000001" "true" "0x$(cat /dev/random | xxd -p | tr -d "\n" | head -c $call_size)"
        echo "R1, R2, Message, Force" >&3
        cast send \
             --legacy \
             --rpc-url $l2_pp_url \
             --value 1 \
             --private-key $target_private_key \
             $bridge_address \
             "bridgeMessage(uint32,address,bool,bytes)" \
             2 "0xFFFF000000000000000000000000000000000001" "true" "0x$(cat /dev/random | xxd -p | tr -d "\n" | head -c $call_size)"
    done

    # local retries=3
    #
    # local agglayer_rpc=$(kurtosis port print $enclave agglayer agglayer)
    #
    # while true ; do
    #     echo "Checking certificate status..." >&3
    #     run cast rpc --rpc-url $agglayer_rpc "interop_getLatestSettledCertificateHeader" 1
    #     assert_success
    #     echo $output >&3
    #     if [[ $(echo $output) != "null" ]]; then
    #         break
    #     fi
    #
    #     retries=$((retries - 1))
    #     if [[ $retries == 0 ]]; then
    #         exit 1
    #     fi
    #     echo "Balance is still 0. Retrying in 10 seconds..." >&3
    #     echo $output >&3
    #     sleep 10
    # done
}

teardown() {
    # kill_kurtosis
    echo "Leftover files in $tmp_dir" >&3
    echo "Teardown" >&3
}
