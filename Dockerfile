FROM --platform=${BUILDPLATFORM} rust:slim-bullseye AS chef
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
    apt-get --no-install-recommends install -y clang cmake libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

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
