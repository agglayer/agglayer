FROM --platform=${BUILDPLATFORM} rust:slim-bookworm AS chef

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

COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --recipe-path recipe.json

COPY --link crates crates
COPY --link Cargo.toml Cargo.toml
COPY --link Cargo.lock Cargo.lock

# Version stamping: no `.git` is copied into the build context, so `vergen`
# cannot derive the version and falls back to a `VERGEN_IDEMPOTENT_OUTPUT`
# placeholder. The caller (CI) computes these from git and passes them in;
# `version()` prefers them over the vergen-derived values.
ARG AGGLAYER_BUILD_DESCRIBE
ARG AGGLAYER_BUILD_TIMESTAMP
ENV AGGLAYER_BUILD_DESCRIBE=${AGGLAYER_BUILD_DESCRIBE}
ENV AGGLAYER_BUILD_TIMESTAMP=${AGGLAYER_BUILD_TIMESTAMP}

RUN cargo build --release --bin agglayer


FROM --platform=${BUILDPLATFORM} debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agglayer /usr/local/bin/

CMD ["/usr/local/bin/agglayer"]
