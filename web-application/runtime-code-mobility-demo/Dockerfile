# Use the official Rust image as the base image
FROM rust:1.75 AS builder

LABEL maintainer="Pratik Poudel<pratikpoudel90@gmail.com>"
ARG BUILD_MODE=debug

# Set the current working directory inside the container
WORKDIR /app

COPY src ./src
COPY Cargo.toml Cargo.lock Config.toml ./

# Build the application
RUN if [ "$BUILD_MODE" = "release" ] ; then cargo build --release; else cargo build; fi

FROM rust:1.75
ARG BUILD_MODE=debug

COPY --from=builder /app/target/${BUILD_MODE}/runtime-code-mobility-demo /usr/local/bin/

# Set the current working directory inside the container
WORKDIR /app
COPY --from=builder /app/Config.toml ./Config.toml
COPY --from=builder /app/src/frontend/static/ ./src/frontend/static/

# Run the application
CMD ["runtime-code-mobility-demo"]
