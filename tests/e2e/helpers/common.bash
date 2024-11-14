#!/usr/bin/env bash

function deploy_contract() {
    local rpc_url="$1"
    local private_key="$2"
    local contract_artifact="$3"

    # Check if rpc_url is available
    if [[ -z "$rpc_url" ]]; then
        echo "Error: rpc_url parameter is not set."
        return 1
    fi

    if [[ ! -f "$contract_artifact" ]]; then
        echo "Error: Contract artifact '$contract_artifact' does not exist."
        return 1
    fi

    # Get the sender address
    local sender=$(cast wallet address "$private_key")
    if [[ $? -ne 0 ]]; then
        echo "Error: Failed to retrieve sender address."
        return 1
    fi

    echo "Attempting to deploy contract artifact '$contract_artifact' to $rpc_url (sender: $sender)" >&3

    # Get bytecode from the contract artifact
    local bytecode=$(jq -r .bytecode "$contract_artifact")
    if [[ -z "$bytecode" || "$bytecode" == "null" ]]; then
        echo "Error: Failed to read bytecode from $contract_artifact"
        return 1
    fi

    # Send the transaction and capture the output
    gas_price=$(cast gas-price --rpc-url "$rpc_url")
    local comp_gas_price=$(bc -l <<< "$gas_price * 1.5" | sed 's/\..*//')
    if [[ $? -ne 0 ]]; then
        echo "Failed to calculate gas price" >&3
        exit 1
    fi
    local cast_output=$(cast send --rpc-url "$rpc_url" \
        --private-key "$private_key" \
        --gas-price $comp_gas_price \
        --legacy \
        --create "$bytecode" \
        2>&1)

    # Check if cast send was successful
    if [[ $? -ne 0 ]]; then
        echo "Error: Failed to send transaction."
        echo "$cast_output"
        return 1
    fi

    echo "Deploy contract output:" >&3
    echo "$cast_output" >&3

    # Extract the contract address from the output
    local deployed_contract_address=$(echo "$cast_output" | grep 'contractAddress' | sed 's/contractAddress\s\+//')
    echo "Deployed contract address: $deployed_contract_address" >&3

    if [[ -z "$deployed_contract_address" ]]; then
        echo "Error: Failed to extract deployed contract address"
        echo "$cast_output"
        return 1
    fi

    if [[ ! "$deployed_contract_address" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        echo "Error: Invalid contract address $deployed_contract_address"
        return 1
    fi

    # Print contract address for return
    echo "$deployed_contract_address"

    return 0
}

function send_tx() {
    # Check if at least 4 arguments are provided
    if [[ $# -lt 4 ]]; then
        echo "Usage: send_tx <rpc_url> <private_key> <receiver> <value_or_function_signature> [<param1> <param2> ...]"
        return 1
    fi

    local rpc_url="$1"               # RPC URL
    local private_key="$2"           # Sender private key
    local receiver_addr="$3"         # Receiver address
    local value_or_function_sig="$4" # Value or function signature

    # Error handling: Ensure the receiver is a valid Ethereum address
    if [[ ! "$receiver_addr" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        echo "Error: Invalid receiver address '$receiver_addr'."
        return 1
    fi

    shift 4             # Shift the first 4 arguments (rpc_url, private_key, receiver_addr, value_or_function_sig)
    local params=("$@") # Collect all remaining arguments as function parameters

    # Get sender address from private key
    local sender
    sender=$(cast wallet address "$private_key") || {
        echo "Error: Failed to extract the sender address."
        return 1
    }

    # Check if the value_or_function_sig is a numeric value (Ether to be transferred)
    if [[ "$value_or_function_sig" =~ ^[0-9]+(\.[0-9]+)?(ether)?$ ]]; then
        # Case: Ether transfer (EOA transaction)
        # Get initial ether balances of sender and receiver
        local sender_addr=$(cast wallet address --private-key "$private_key")
        local sender_initial_balance receiver_initial_balance
        sender_initial_balance=$(cast balance "$sender_addr" --ether --rpc-url "$rpc_url") || return 1
        receiver_initial_balance=$(cast balance "$receiver_addr" --ether --rpc-url "$rpc_url") || return 1
        
        send_eoa_transaction "$private_key" "$receiver_addr" "$value_or_function_sig" "$sender_addr" "$sender_initial_balance" "$receiver_initial_balance"
    else
        # Case: Smart contract interaction (contract interaction with function signature and parameters)
        send_smart_contract_transaction "$private_key" "$receiver_addr" "$value_or_function_sig" "${params[@]}"
    fi
}

function send_eoa_transaction() {
    local private_key="$1"
    local receiver_addr="$2"
    local value="$3"
    local sender="$4"
    local sender_initial_balance="$5"
    local receiver_initial_balance="$6"

    echo "Sending EOA transaction (from: $sender, rpc url: $rpc_url) to: $receiver_addr with value: $value" >&3

    # Send transaction via cast
    local cast_output tx_hash
    gas_price=$(cast gas-price --rpc-url "$rpc_url")
    local comp_gas_price=$(bc -l <<< "$gas_price * 1.5" | sed 's/\..*//')
    if [[ $? -ne 0 ]]; then
        echo "Failed to calculate gas price" >&3
        exit 1
    fi
    echo "cast send --gas-price $comp_gas_price --rpc-url $rpc_url --private-key $private_key $receiver_addr --value $value --legacy" >&3
    cast_output=$(cast send --gas-price $comp_gas_price --rpc-url "$rpc_url" --private-key "$private_key" "$receiver_addr" --value "$value" --legacy 2>&1)
    if [[ $? -ne 0 ]]; then
        echo "Error: Failed to send transaction. Output:"
        echo "$cast_output"
        return 1
    fi

    tx_hash=$(extract_tx_hash "$cast_output")
    [[ -z "$tx_hash" ]] && {
        echo "Error: Failed to extract transaction hash."
        return 1
    }

    check_balances "$sender" "$receiver_addr" "$value" "$tx_hash" "$sender_initial_balance" "$receiver_initial_balance"
    if [[ $? -ne 0 ]]; then
        echo "Error: Balance not updated correctly."
        return 1
    fi

    echo "Transaction successful (transaction hash: $tx_hash)"
}

function send_smart_contract_transaction() {
    local private_key="$1"
    local receiver_addr="$2"
    local function_sig="$3"
    shift 3
    local params=("$@")

    echo "Sending smart contract transaction to $receiver_addr with function signature: '$function_sig' and params: ${params[*]}" >&3

    # Send the smart contract interaction using cast
    local cast_output tx_hash
    gas_price=$(cast gas-price --rpc-url "$rpc_url")
    local comp_gas_price=$(bc -l <<< "$gas_price * 1.5" | sed 's/\..*//')
    if [[ $? -ne 0 ]]; then
        echo "Failed to calculate gas price" >&3
        exit 1
    fi
    cast_output=$(cast send "$receiver_addr" --rpc-url "$rpc_url" --private-key "$private_key" --gas-price $comp_gas_price --legacy "$function_sig" "${params[@]}" 2>&1)
    if [[ $? -ne 0 ]]; then
        echo "Error: Failed to send transaction. Output:"
        echo "$cast_output"
        return 1
    fi

    tx_hash=$(extract_tx_hash "$cast_output")
    [[ -z "$tx_hash" ]] && {
        echo "Error: Failed to extract transaction hash."
        return 1
    }

    echo "Transaction successful (transaction hash: $tx_hash)"
}

function extract_tx_hash() {
    local cast_output="$1"
    echo "$cast_output" | grep 'transactionHash' | awk '{print $2}' | tail -n 1
}

function query_contract() {
    local rpc_url="$1"       # RPC URL
    local addr="$2"          # Contract address
    local funcSignature="$3" # Function signature
    shift 3                  # Shift past the first 3 arguments
    local params=("$@")      # Collect remaining arguments as parameters array

    echo "Querying state of $addr account (RPC URL: $rpc_url) with function signature: '$funcSignature' and params: ${params[*]}" >&3

    # Check if rpc url is available
    if [[ -z "$rpc_url" ]]; then
        echo "Error: rpc_url parameter is not provided."
        return 1
    fi

    # Check if the contract address is valid
    if [[ ! "$addr" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        echo "Error: Invalid contract address '$addr'."
        return 1
    fi

    # Call the contract using `cast call`
    local result
    result=$(cast call --rpc-url "$rpc_url" "$addr" "$funcSignature" "${params[@]}" 2>&1)

    # Check if the call was successful
    if [[ $? -ne 0 ]]; then
        echo "Error: Failed to query contract."
        echo "$result"
        return 1
    fi

    # Return the result (contract query response)
    echo "$result"

    return 0
}

function check_balances() {
    local sender="$1"
    local receiver="$2"
    local amount="$3"
    local tx_hash="$4"
    local sender_initial_balance="$5"
    local receiver_initial_balance="$6"

    # Ethereum address regex: 0x followed by 40 hexadecimal characters
    if [[ ! "$sender" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        echo "Error: Invalid sender address '$sender'."
        return 1
    fi

    if [[ ! "$receiver" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        echo "Error: Invalid receiver address '$receiver'."
        return 1
    fi

    # Transaction hash regex: 0x followed by 64 hexadecimal characters
    if [[ ! "$tx_hash" =~ ^0x[a-fA-F0-9]{64}$ ]]; then
        echo "Error: Invalid transaction hash: $tx_hash".
        return 1
    fi

    local sender_final_balance=$(cast balance "$sender" --ether --rpc-url "$rpc_url") || return 1
    local tx_output=$(cast tx "$tx_hash" --rpc-url "$rpc_url")
    local gas_used=$(tx_output | grep '^gas ' | awk '{print $2}')
    local gas_price=$(tx_output | grep '^gasPrice' | awk '{print $2}')
    local gas_fee=$(echo "$gas_used * $gas_price" | bc)
    local gas_fee_in_ether=$(cast to-unit "$gas_fee" ether)

    local sender_balance_change=$(echo "$sender_initial_balance - $sender_final_balance" | bc)
    echo "Sender balance changed by: '$sender_balance_change' wei"
    echo "Gas fee paid: '$gas_fee_in_ether' ether"

    local receiver_final_balance=$(cast balance "$receiver" --ether --rpc-url "$rpc_url") || return 1
    local receiver_balance_change=$(echo "$receiver_final_balance - $receiver_initial_balance" | bc)
    echo "Receiver balance changed by: '$receiver_balance_change' wei"

    # Trim 'ether' suffix from amount to get the numeric part
    local value_in_ether=$(echo "$amount" | sed 's/ether$//')

    if ! echo "$receiver_balance_change == $value_in_ether" | bc -l; then
        echo "Error: receiver balance updated incorrectly. Expected: $value_in_ether, Actual: $receiver_balance_change"
        return 1
    fi

    # Calculate expected sender balance change
    local expected_sender_change=$(echo "$value_in_ether + $gas_fee_in_ether" | bc)
    if ! echo "$sender_balance_change == $expected_sender_change" | bc -l; then
        echo "Error: sender balance updated incorrectly. Expected: $expected_sender_change, Actual: $sender_balance_change"
        return 1
    fi
}

function verify_balance() {
    local rpc_url="$1"             # RPC URL
    local token_addr="$2"          # gas token contract address
    local account="$3"             # account address
    local initial_balance_wei="$4" # initial balance in Wei (decimal)
    local ether_amount="$5"        # amount to be added (in Ether, decimal)

    # Trim 'ether' from ether_amount if it exists
    ether_amount=$(echo "$ether_amount" | sed 's/ether//')
    local amount_wei=$(cast --to-wei "$ether_amount")
    
    # Get final balance in wei (after the operation)
    local final_balance_wei
    if [[ $token_addr == "0x0000000000000000000000000000000000000000" ]]; then
        final_balance_wei=$(cast balance "$account" --rpc-url "$rpc_url" | awk '{print $1}')
    else
        final_balance_wei=$(cast call --rpc-url "$rpc_url" "$token_addr" "$balance_of_fn_sig" "$destination_addr" | awk '{print $1}')
    fi
    echo "Final balance of $account in $rpc_url: $final_balance_wei wei" >&3

    # Calculate expected final balance (initial_balance + amount)
    local expected_final_balance_wei=$(echo "$initial_balance_wei + $amount_wei" | bc)

    # Check if final_balance matches the expected final balance
    if [ "$(echo "$final_balance_wei == $expected_final_balance_wei" | bc)" -eq 1 ]; then
        echo "✅ Balance verification successful: final balance is correct."
    else
        echo "❌ Balance verification failed: expected $expected_final_balance_wei but got $final_balance_wei." >&3
        exit 1
    fi
}

function mint_erc20_tokens() {
    local rpc_url="$1"            # The L1 RPC URL
    local erc20_token_addr="$2"   # The gas token contract address
    local minter_private_key="$3" # The minter private key
    local receiver="$4"           # The receiver address (for minted tokens)
    local tokens_amount="$5"      # The amount of tokens to transfer (e.g., "0.1ether")

    # Query the erc20 token balance of the sender
    run query_contract "$rpc_url" "$erc20_token_addr" "$balance_of_fn_sig" "$sender_addr"
    assert_success
    local erc20_token_balance=$(echo "$output" | tail -n 1)

    # Log the account's current gas token balance
    echo "Initial account balance: $erc20_token_balance wei" >&3

    # Convert tokens_amount to Wei for comparison
    local wei_amount=$(cast --to-unit "$tokens_amount" wei)

    # Mint the required tokens by sending a transaction
    run send_tx "$rpc_url" "$minter_private_key" "$erc20_token_addr" "$mint_fn_sig" "$receiver" "$tokens_amount"
    assert_success
}
