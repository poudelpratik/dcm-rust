# Use the official Rust image as the base image
FROM rust:1.74 AS builder

LABEL autodelete="true"
LABEL maintainer="Pratik Poudel<pratikpoudel90@gmail.com>"

# Set the current working directory inside the container
WORKDIR /usr/src/app

# Create a new user inside the container
#ARG USER=wasm_generator
#ARG GROUP=wasm_generator
#ARG UID=1000
#ARG GID=1000
#RUN groupadd -g $GID $GROUP \
#  && useradd -m -g $GID -u $UID $USER

COPY src ./src
# Copy the Cargo.toml, Cargo.lock, and Config.toml files
COPY Cargo.toml Cargo.lock Config.toml ./


# Build the application
RUN cargo build --release

FROM rust:1.74

# Copy the binary from the builder stage. Change this from debug to release later, currently it is in debug for development purposes for its reduced compilation time.
COPY --from=builder /usr/src/app/target/release/wasm-generator /usr/local/bin/

# Copy the Config.toml from the builder stage
COPY --from=builder /usr/src/app/Config.toml /usr/local/bin/

# Install the rust-analyzer to be used by LspClient
RUN rustup component add rust-analyzer
# Install the rustfmt tool for formatting the fragment code
RUN rustup component add rustfmt
# Install the rust-wasm toolchain
RUN rustup target add wasm32-unknown-unknown
#RUN rustup target add wasm32-wasi
#RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
# Install the binaryen for wasm-opt
RUN apt update && \
    apt install -y binaryen && \
    rm -rf /var/lib/apt/lists/*

#USER $USER

# Set the current working directory for runtime inside the container. This is specially so that the Config.toml file can be found.
WORKDIR /usr/local/bin/

# Run the application
CMD ["wasm-generator"]
