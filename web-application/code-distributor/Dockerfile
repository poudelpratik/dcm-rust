# Use the official Rust image as the base image
FROM rust:1.74 AS builder

LABEL maintainer="Pratik Poudel<pratikpoudel90@gmail.com>"
ARG BUILD_MODE=debug

# Set the current working directory inside the container
WORKDIR /app

COPY src ./src
COPY Cargo.toml Cargo.lock Config.toml ./
COPY fragments ./fragments

# Build the application
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release; else cargo build; fi

FROM rust:1.74
ARG BUILD_MODE=debug

COPY --from=builder /app/target/${BUILD_MODE}/code-distributor /usr/local/bin/

# Set the current working directory inside the container
WORKDIR /app
COPY --from=builder /app/fragments ./fragments
COPY --from=builder /app/Config.toml ./

# Run the application
CMD ["code-distributor"]
