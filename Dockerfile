FROM rust:slim-bullseye as builder

RUN mkdir -p src && echo "fn main() {}" > src/main.rs

COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release

COPY src ./src
RUN touch src/main.rs

RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder ./target/release/agglayer /usr/local/bin/agglayer

CMD ["agglayer"]
