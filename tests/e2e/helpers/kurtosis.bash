#!/bin/bash

setup_kurtosis() {
    bats_load_library 'bats-support'
    bats_load_library 'bats-assert'

    readonly kurtosis="$(which kurtosis)"
    readonly kurtosis_path="$1"
}

start_network() {
    bats_load_library 'bats-support'
    bats_load_library 'bats-assert'

    local args_file="$2"

    $kurtosis run --enclave $enclave --args-file $args_file $kurtosis_path >&3
}

kill_kurtosis() {
    $kurtosis clean --all >&3
}
