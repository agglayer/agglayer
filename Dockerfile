FROM --platform=${BUILDPLATFORM} rust:slim-bullseye AS chef

ARG PROTOC_VERSION=28.2

USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner

COPY --link crates crates
COPY --link xtask xtask
COPY --link Cargo.toml Cargo.toml
COPY --link Cargo.lock Cargo.lock

RUN cargo chef prepare --recipe-path recipe.json --bin agglayer

FROM chef AS builder

RUN apt-get update && \
    apt-get --no-install-recommends install -y clang cmake curl libssl-dev pkg-config unzip && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

RUN ARCHITECTURE=$(uname -m | sed -e "s/arm64/arm_64/g" | sed -e "s/aarch64/aarch_64/g") \
    && curl -LOs "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip" \
    && unzip -o "protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip" -d /usr/local bin/protoc \
    && unzip -o "protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip" -d /usr/local 'include/*' \
    && chmod +x "/usr/local/bin/protoc" \
    && rm "protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip"


COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --recipe-path recipe.json

COPY --link crates crates
COPY --link Cargo.toml Cargo.toml
COPY --link Cargo.lock Cargo.lock

RUN cargo build --release --bin agglayer

FROM --platform=${BUILDPLATFORM} debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/agglayer /usr/local/bin/

CMD ["/usr/local/bin/agglayer"]
