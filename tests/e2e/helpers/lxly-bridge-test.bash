#!/usr/bin/env bash
# Error code reference https://hackmd.io/WwahVBZERJKdfK3BbKxzQQ
function bridgeAsset() {
    local token_addr="$1"
    local rpc_url="$2"
    readonly bridge_sig='bridgeAsset(uint32,address,uint256,address,bool,bytes)'

    if [[ $token_addr == "0x0000000000000000000000000000000000000000" ]]; then
        echo "The ETH balance for sender "$sender_addr":" >&3
        cast balance -e --rpc-url $rpc_url $sender_addr >&3
    else
        echo "The "$token_addr" token balance for sender "$sender_addr":" >&3
        echo "cast call --rpc-url $rpc_url $token_addr \"$balance_of_fn_sig\" $sender_addr" >&3
        balance_wei=$(cast call --rpc-url "$rpc_url" "$token_addr" "$balance_of_fn_sig" "$sender_addr" | awk '{print $1}')
        echo "$(cast --from-wei "$balance_wei")" >&3
    fi

    echo "Attempting to deposit $amount [wei] to $destination_addr, token $token_addr (sender=$sender_addr, network id=$destination_net, rpc url=$rpc_url)" >&3

    if [[ $dry_run == "true" ]]; then
        cast calldata $bridge_sig $destination_net $destination_addr $amount $token_addr $is_forced $meta_bytes
    else
        if [[ $token_addr == "0x0000000000000000000000000000000000000000" ]]; then
            echo "cast send --legacy --private-key $sender_private_key --value $amount --rpc-url $rpc_url $bridge_addr $bridge_sig $destination_net $destination_addr $amount $token_addr $is_forced $meta_bytes" >&3
            cast send --legacy --private-key $sender_private_key --value $amount --rpc-url $rpc_url $bridge_addr $bridge_sig $destination_net $destination_addr $amount $token_addr $is_forced $meta_bytes
        else
            echo "cast send --legacy --private-key $sender_private_key --rpc-url $rpc_url $bridge_addr \"$bridge_sig\" $destination_net $destination_addr $amount $token_addr $is_forced $meta_bytes"
            cast send --legacy --private-key $sender_private_key --rpc-url $rpc_url $bridge_addr $bridge_sig $destination_net $destination_addr $amount $token_addr $is_forced $meta_bytes
        fi
    fi
}

function claim() {
    local destination_rpc_url="$1"
    readonly claim_sig="claimAsset(bytes32[32],bytes32[32],uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)"
    readonly bridge_deposit_file=$(mktemp)
    readonly claimable_deposit_file=$(mktemp)
    echo "Getting full list of deposits" >&3
    curl -s "$bridge_api_url/bridges/$destination_addr?limit=100&offset=0" | jq '.' | tee $bridge_deposit_file

    echo "Looking for claimable deposits" >&3
    jq '[.deposits[] | select(.ready_for_claim == true and .claim_tx_hash == "" and .dest_net == '$destination_net')]' $bridge_deposit_file | tee $claimable_deposit_file
    readonly claimable_count=$(jq '. | length' $claimable_deposit_file)
    echo "Found $claimable_count claimable deposits" >&3

    if [[ $claimable_count == 0 ]]; then
        echo "We have no claimable deposits at this time" >&3
        exit 1
    fi

    echo "We have $claimable_count claimable deposits on network $destination_net. Let's get this party started." >&3
    readonly current_deposit=$(mktemp)
    readonly current_proof=$(mktemp)
    local gas_price_factor=1
    while read deposit_idx; do
        echo "Starting claim for tx index: "$deposit_idx >&3
        echo "Deposit info:" >&3
        jq --arg idx $deposit_idx '.[($idx | tonumber)]' $claimable_deposit_file | tee $current_deposit >&3

        curr_deposit_cnt=$(jq -r '.deposit_cnt' $current_deposit)
        curr_network_id=$(jq -r '.network_id' $current_deposit)
        curl -s "$bridge_api_url/merkle-proof?deposit_cnt=$curr_deposit_cnt&net_id=$curr_network_id" | jq '.' | tee $current_proof

        in_merkle_proof="$(jq -r -c '.proof.merkle_proof' $current_proof | tr -d '"')"
        in_rollup_merkle_proof="$(jq -r -c '.proof.rollup_merkle_proof' $current_proof | tr -d '"')"
        in_global_index=$(jq -r '.global_index' $current_deposit)
        in_main_exit_root=$(jq -r '.proof.main_exit_root' $current_proof)
        in_rollup_exit_root=$(jq -r '.proof.rollup_exit_root' $current_proof)
        in_orig_net=$(jq -r '.orig_net' $current_deposit)
        in_orig_addr=$(jq -r '.orig_addr' $current_deposit)
        in_dest_net=$(jq -r '.dest_net' $current_deposit)
        in_dest_addr=$(jq -r '.dest_addr' $current_deposit)
        in_amount=$(jq -r '.amount' $current_deposit)
        in_metadata=$(jq -r '.metadata' $current_deposit)

        if [[ $dry_run == "true" ]]; then
            cast calldata $claim_sig "$in_merkle_proof" "$in_rollup_merkle_proof" $in_global_index $in_main_exit_root $in_rollup_exit_root $in_orig_net $in_orig_addr $in_dest_net $in_dest_addr $in_amount $in_metadata
        else
            local comp_gas_price=$(bc -l <<< "$gas_price * 1.5" | sed 's/\..*//')
            if [[ $? -ne 0 ]]; then
                echo "Failed to calculate gas price" >&3
                exit 1
            fi
            
            echo "cast send --legacy --gas-price $comp_gas_price --rpc-url $destination_rpc_url --private-key $sender_private_key $bridge_addr \"$claim_sig\" \"$in_merkle_proof\" \"$in_rollup_merkle_proof\" $in_global_index $in_main_exit_root $in_rollup_exit_root $in_orig_net $in_orig_addr $in_dest_net $in_dest_addr $in_amount $in_metadata" >&3
            cast send --legacy --gas-price $comp_gas_price --rpc-url $destination_rpc_url --private-key $sender_private_key $bridge_addr "$claim_sig" "$in_merkle_proof" "$in_rollup_merkle_proof" $in_global_index $in_main_exit_root $in_rollup_exit_root $in_orig_net $in_orig_addr $in_dest_net $in_dest_addr $in_amount $in_metadata
        fi

    done < <(seq 0 $((claimable_count - 1)))
}

function wait_for_claim() {
    local timeout="$1"         # timeout (in seconds)
    local claim_frequency="$2" # claim frequency (in seconds)
    local destination_rpc_url="$3" # destination rpc url
    local start_time=$(date +%s)
    local end_time=$((start_time + timeout))

    while true; do
        local current_time=$(date +%s)
        if ((current_time > end_time)); then
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] ❌ Exiting... Timeout reached!"
            exit 1
        fi

        run claim $destination_rpc_url
        if [ $status -eq 0 ]; then
            break
        fi

        sleep "$claim_frequency"
    done
}
