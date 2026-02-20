#!/usr/bin/env bash
set -euixo pipefail

time="$1"
fuzzers=(
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_certificate_id"
    "agglayer-grpc-types/compat::v1::tests::fuzz_parser_epoch_configuration"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_certificate_id"
    "agglayer-grpc-types/compat::v1::tests::fuzz_round_trip_epoch_configuration"
)

printf '%s\0' "${fuzzers[@]}" | parallel --null --bar --joblog fuzz.log bash -c "
    crate=\"\$(echo {} | cut -d/ -f1)\"
    target=\"\$(echo {} | cut -d/ -f2)\"
    cargo bolero test --rustc-bootstrap -p \"\$crate\" --all-features \"\$target\" -T '$time'
"
