# Use the official Rust image as the base image
FROM rust:1.75 AS builder

LABEL autodelete="true"
LABEL maintainer="Pratik Poudel<pratikpoudel90@gmail.com>"
ARG BUILD_MODE=debug

# Set the current working directory inside the container
WORKDIR /app

COPY src ./src
# Copy the Cargo.toml, Cargo.lock, and Config.toml files
COPY Cargo.toml Cargo.lock Config.toml ./


# Build the application
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release; else cargo build; fi

FROM rust:1.75
ARG BUILD_MODE=debug
# Copy the binary from the builder stage. Change this from debug to release later, currently it is in debug for development purposes for its reduced compilation time.
COPY --from=builder /app/target/${BUILD_MODE}/wasm-generator /usr/local/bin/

# Copy the Config.toml from the builder stage
COPY --from=builder /app/Config.toml /usr/local/bin/

# Install the rust-analyzer to be used by LspClient
RUN rustup component add rust-analyzer
# Install the rustfmt tool for formatting the fragment code
RUN rustup component add rustfmt
# Install the rust-wasm toolchain
RUN rustup target add wasm32-unknown-unknown
# Install the binaryen tool for wasm-opt
RUN apt update && \
    apt install -y binaryen && \
    rm -rf /var/lib/apt/lists/*

# Install the build-essential package for the linker
RUN apt install build-essential

#USER $USER

# Set the current working directory for runtime inside the container. This is specially so that the Config.toml file can be found.
WORKDIR /usr/local/bin/

# Run the application
CMD ["wasm-generator"]
