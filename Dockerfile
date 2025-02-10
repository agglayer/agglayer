FROM --platform=${BUILDPLATFORM} rust:slim-bullseye AS chef

ARG CIRCUIT_ARTIFACTS_URL_BASE=https://sp1-circuits.s3-us-east-2.amazonaws.com
ARG CIRCUIT_TYPE=plonk
ARG CIRCUIT_VERSION=v4.0.0-rc.3
ARG PROTOC_VERSION=28.2
ARG CHEF_VERSION=0.1.68

USER root
RUN cargo install --version ${CHEF_VERSION} cargo-chef
WORKDIR /app

FROM chef AS planner

COPY --link crates crates
# Needed for cargo-chef to build, but not use during the compilation due to `--bin agglayer`
COPY --link tests/integrations tests/integrations
COPY --link Cargo.toml Cargo.toml
COPY --link Cargo.lock Cargo.lock

RUN cargo chef prepare --recipe-path recipe.json --bin agglayer

FROM --platform=${BUILDPLATFORM} golang:1.22 AS go-builder

FROM chef AS builder

RUN apt-get update && \
    apt-get --no-install-recommends install -y clang cmake curl libssl-dev tar pkg-config unzip && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

RUN ARCHITECTURE=$(uname -m | sed -e "s/arm64/arm_64/g" | sed -e "s/aarch64/aarch_64/g") \
    && curl -LOs --proto "=https" "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip" \
    && unzip -o "protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip" -d /usr/local bin/protoc \
    && unzip -o "protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip" -d /usr/local 'include/*' \
    && chmod +x "/usr/local/bin/protoc" \
    && rm "protoc-${PROTOC_VERSION}-linux-$ARCHITECTURE.zip"

# Install Go 1.22
COPY --from=go-builder /usr/local/go /usr/local/go
ENV PATH="/usr/local/go/bin:$PATH"

COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --recipe-path recipe.json

RUN mkdir -p /root/.sp1/circuits/${CIRCUIT_TYPE}/${CIRCUIT_VERSION}
RUN curl -s -o /tmp/circuits.tar.gz ${CIRCUIT_ARTIFACTS_URL_BASE}/${CIRCUIT_VERSION}-${CIRCUIT_TYPE}.tar.gz \
    && tar -Pxzf/tmp/circuits.tar.gz -C /root/.sp1/circuits/${CIRCUIT_TYPE}/${CIRCUIT_VERSION}

COPY --link crates crates
COPY --link Cargo.toml Cargo.toml
COPY --link Cargo.lock Cargo.lock

RUN cargo build --release --bin agglayer


FROM --platform=${BUILDPLATFORM} debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/agglayer /usr/local/bin/
COPY --from=builder /root/.sp1/circuits /root/.sp1/circuits

CMD ["/usr/local/bin/agglayer"]
