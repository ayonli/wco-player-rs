
FROM --platform=linux/arm64 rust:1.92-alpine as builder

# Install build dependencies (Alpine uses apk package manager)
# linux-headers is required for memmap crate compilation
# For OpenSSL static linking, we need both openssl-dev and openssl-libs-static
RUN apk add --no-cache \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    ca-certificates \
    curl \
    build-base \
    git \
    linux-headers \
    nodejs \
    npm

# Verify Rust version
RUN rustc --version && cargo --version

# Install Rust stable toolchain and WASM target
RUN rustup toolchain install stable
RUN rustup target add wasm32-unknown-unknown

# Install Dioxus CLI and wasm-bindgen-cli
RUN cargo install --locked dioxus-cli wasm-bindgen-cli

# Install wasm-opt (binaryen) from source
# Pre-built binaries are typically glibc-based and won't work on Alpine (musl)
# This is required for WASM optimization in dx build
# Disable -Werror to avoid GCC 15.2.0 false positives with -Wmaybe-uninitialized
# Copy both binary and shared library, set LD_LIBRARY_PATH
RUN apk add --no-cache cmake python3 ninja git && \
    git clone --recursive --depth 1 --branch version_116 https://github.com/WebAssembly/binaryen.git /tmp/binaryen && \
    cd /tmp/binaryen && \
    cmake -DCMAKE_BUILD_TYPE=Release -G Ninja -DBUILD_TESTS=OFF \
    -DCMAKE_CXX_FLAGS="-Wno-error -Wno-maybe-uninitialized" . && \
    ninja wasm-opt && \
    cp bin/wasm-opt /usr/local/bin/ && \
    cp lib/libbinaryen.so /usr/local/lib/ && \
    ldconfig && \
    rm -rf /tmp/binaryen && \
    apk del cmake python3 ninja git && \
    wasm-opt --version

# Set NO_DOWNLOADS=1 to make dx build use PATH tools instead of downloading
# This ensures dx uses the musl-compatible tools we just installed
ENV NO_DOWNLOADS=1

# Set working directory
WORKDIR /app

# Copy package.json for Node.js dependencies
COPY package.json package-lock.json* ./

# Install Node.js dependencies
RUN npm ci || npm install

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY packages/web/Cargo.toml ./packages/web/
COPY packages/ui/Cargo.toml ./packages/ui/
COPY packages/api/Cargo.toml ./packages/api/
COPY packages/wco/Cargo.toml ./packages/wco/

# Create dummy source files to build dependencies (for Docker layer caching)
# This allows us to cache compiled dependencies separately from source code changes
RUN mkdir -p packages/web/src packages/ui/src packages/api/src packages/wco/src && \
    echo "fn main() {}" > packages/web/src/main.rs && \
    echo "// dummy" > packages/ui/src/lib.rs && \
    echo "// dummy" > packages/api/src/lib.rs && \
    echo "// dummy" > packages/wco/src/lib.rs

# We build for the server target to cache server dependencies
RUN cargo build --package web --features server --release || true

# Remove dummy source files
RUN rm -rf packages/web/src packages/ui/src packages/api/src packages/wco/src

# Copy actual source code
COPY packages/ ./packages/

# Build web application
RUN dx build --package web --platform web --release
