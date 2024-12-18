# Build arguments specifying the toolchain version and location
ARG RUST_VERSION SP1_TOOLCHAIN_HASH SP1_VERSION SP1_ARCH
ARG SP1_BASE_URL=https://github.com/succinctlabs/rust/releases/download

# Base image with rust compiler & tools
FROM docker.io/library/rust:${RUST_VERSION}

# Import arguments
ARG RUST_VERSION SP1_TOOLCHAIN_HASH SP1_VERSION SP1_TARGET SP1_BASE_URL SP1_ARCH

# Mark the image with arguably useful metadata
LABEL \
    agglayer.sp1.toolchain.version="$SP1_VERSION" \
    agglayer.sp1.toolchain.digest="$SP1_TOOLCHAIN_HASH" \
    agglayer.rust.version="$RUST_VERSION"

WORKDIR /sp1

# Fetch the toolchain archive & check it
ADD --checksum=sha256:$SP1_TOOLCHAIN_HASH \
    "$SP1_BASE_URL/$SP1_VERSION/rust-toolchain-$SP1_ARCH-unknown-linux-gnu.tar.gz" \
    /sp1/install/rust-toolchain.tar.gz

# Unpack and install the toolchain
RUN : \
    && mkdir -p sp1-toolchain \
    && tar -xzf /sp1/install/rust-toolchain.tar.gz -C /sp1/sp1-toolchain
RUN : \
    && rustup toolchain link succinct /sp1/sp1-toolchain \
    && rustup default succinct

# Cleanup
RUN rm -rf /sp1/install

# Drop root priviledges
RUN useradd --create-home --home-dir=/sp1 sp1
USER sp1
