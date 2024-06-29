# Step 1: Build the application
FROM rust:alpine as builder

RUN apk update && apk add --no-cache \
    openssl \
    openssl-dev \
    musl-dev \
    pkgconfig \
    build-base

# Create app directory
WORKDIR /app

# Copy the source code into the container
COPY Cargo.toml ./
COPY src ./src


# Build the application
RUN cargo build --bin blob_proxy --release --features=blob_proxy

# Step 2: Create the runtime image
FROM alpine:latest

# Install necessary runtime dependencies
RUN apk add --no-cache libgcc libstdc++ openssl ca-certificates

# Create a user to run the application
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Copy the built application from the builder
COPY --from=builder /app/target/release/blob_proxy /usr/local/bin/blob_proxy


# Change ownership of the application binary
RUN chown appuser:appgroup /usr/local/bin/blob_proxy



# Switch to the non-root user
USER appuser

EXPOSE 8888
# Run the application
ENTRYPOINT ["blob_proxy"]