#!/bin/sh
cargo run --bin convertor -- ./dataold/n15-cert_h0.json data/n15-cert_h0.json
cargo run --bin convertor -- ./dataold/n15-cert_h1.json data/n15-cert_h1.json
cargo run --bin convertor -- ./dataold/n15-cert_h2.json data/n15-cert_h2.json
cargo run --bin convertor -- ./dataold/n15-cert_h3.json data/n15-cert_h3.json
