FROM rust:alpine3.21

# Install required packages for downloading and building OpenSSL.
RUN apk update && \
    apk add --no-cache \
      wget \
      tar \
      build-base \
      perl

# Download and extract the precompiled aarch64 musl cross compiler.
RUN wget https://musl.cc/aarch64-linux-musl-cross.tgz && \
    echo "a6bb806af217a91cf575e15163e8b12b  aarch64-linux-musl-cross.tgz" | md5sum -c - && \
    tar -xzf aarch64-linux-musl-cross.tgz -C /usr/local && \
    rm aarch64-linux-musl-cross.tgz

# Add the cross compiler binaries to PATH.
ENV PATH="/usr/local/aarch64-linux-musl-cross/bin:${PATH}"

# Add the musl target for aarch64.
RUN rustup target add aarch64-unknown-linux-musl

# Build OpenSSL for aarch64 with musl.
RUN wget https://www.openssl.org/source/openssl-1.1.1l.tar.gz && \
    echo "ac0d4387f3ba0ad741b0580dd45f6ff3  openssl-1.1.1l.tar.gz" | md5sum -c - && \
    tar -xzf openssl-1.1.1l.tar.gz && \
    cd openssl-1.1.1l && \
    ./Configure linux-aarch64 \
      -static \
      --cross-compile-prefix=aarch64-linux-musl- \
      --prefix=/usr/local/openssl-aarch64 \
      no-shared && \
    make -j"$(nproc)" && \
    make install_sw && \
    cd .. && \
    rm -rf openssl-1.1.1l openssl-1.1.1l.tar.gz

# Tell Cargo (and build scripts) where to find OpenSSL.
ENV OPENSSL_DIR=/usr/local/openssl-aarch64 \
    OPENSSL_LIB_DIR=/usr/local/openssl-aarch64/lib \
    OPENSSL_INCLUDE_DIR=/usr/local/openssl-aarch64/include \
    PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /app

# Use the aarch64 musl cross linker when building.
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc

# Default command: build your Rust application for aarch64/musl
CMD ["cargo", "build", "--release", "--target", "aarch64-unknown-linux-musl"]
