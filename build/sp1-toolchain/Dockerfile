ARG RUST_VERSION=1.81.0

FROM rust:${RUST_VERSION}

# Drop root priviledges
RUN useradd --create-home --home-dir=/sp1 sp1
USER sp1
WORKDIR /sp1

# Build arguments specifying the toolchain version and location
ARG RUST_VERSION \
    TOOLCHAIN_HASH=sha256:f8b978b44fdedfd9f3dfea127ee11f13ee2fdaaffdea2619489dff7bbe791cce \
    SP1_VERSION=$RUST_VERSION \
    TARGET=x86_64-unknown-linux-gnu \
    SP1_BASE_URL=https://github.com/succinctlabs/rust/releases/download

# Mark the image with arguably useful metadata
LABEL \
    agglayer.sp1.toolchain.version="$SP1_VERSION" \
    agglayer.sp1.toolchain.digest="$TOOLCHAIN_HASH" \
    agglayer.rust.version="$RUST_VERSION"

# Fetch the toolchain archive
ADD --checksum=$TOOLCHAIN_HASH --chown=sp1 \
    $SP1_BASE_URL/v$SP1_VERSION/rust-toolchain-$TARGET.tar.gz \
    /sp1/install/rust-toolchain.tar.gz

# Unpack the toolchain
RUN : \
    && mkdir -p sp1-toolchain \
    && tar -xzf /sp1/install/rust-toolchain.tar.gz -C /sp1/sp1-toolchain

# Install the succinct Rust toolchain
RUN : \
    && rustup toolchain link succinct /sp1/sp1-toolchain \
    && rustup default succinct

# Cleanup
RUN rm -rf /sp1/install
