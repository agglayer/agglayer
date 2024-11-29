cdk_check_virtual_batch_increase() {
    _loop_over_batch $1 $2 zkevm_virtualBatchNumber
}

cdk_check_verified_batch_increase() {
    _loop_over_batch $1 $2 zkevm_verifiedBatchNumber
}

_loop_over_batch() {
    local batch_number=$(cast to-dec "$(cast rpc --rpc-url $1 $3 | sed 's/"//g')")

    for i in `seq 1 $2`; do
        sleep 10

        new="$(cast to-dec "$(cast rpc --rpc-url $1 $3 | sed 's/"//g')")"

        if ((new > batch_number)); then
            exit 0
        fi

    done

    exit 1
}
