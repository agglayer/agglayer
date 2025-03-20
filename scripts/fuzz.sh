#!/usr/bin/env bash
set -euixo pipefail

time="$1"
fuzzers=(
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_address"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_aggchain_data"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_bridge_exit"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_certificate_id"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_claim_from_mainnet"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_claim_from_rollup"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_digest"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_epoch_configuration"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_global_index"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_imported_bridge_exit"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_l1_info_tree_leaf_with_context"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_l1_info_tree_leaf_inner"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_merkle_proof"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_token_info"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_u256"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_address"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_aggchain_data"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_bridge_exit"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_certificate_id"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_claim_from_mainnet"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_claim_from_rollup"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_digest"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_epoch_configuration"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_global_index"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_imported_bridge_exit"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_l1_info_tree_leaf_with_context"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_l1_info_tree_leaf_inner"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_merkle_proof"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_token_info"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_u256"
)

printf '%s\0' "${fuzzers[@]}" | parallel --null --bar --joblog fuzz.log bash -c '
    crate="$(echo {} | cut -d/ -f1)"
    target="$(echo {} | cut -d/ -f2)"
    cargo bolero test --rustc-bootstrap -p "$crate" --all-features "$target" -T '"$time"'
'
