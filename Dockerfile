# Use the official Rust image as the base image
FROM rust:latest AS build

# Create a new directory for the app
WORKDIR /app

# Copy the source code
COPY . .

# Install dependencies and build the application
RUN apt-get update && \
    apt-get install -y libpq-dev && \
    cargo install diesel_cli --no-default-features --features postgres && \
    cargo build --release

# Use a Rust image for the final stage
FROM rust:latest

# Copy the built binary from the build stage
COPY --from=build /app/target/release/ethereum-fetcher /usr/local/bin/ethereum-fetcher

ENV API_PORT=${API_PORT}
ENV DB_CONNECTION_URL=${DB_CONNECTION_URL}
ENV ETH_NODE_URL=${ETH_NODE_URL}
ENV JWT_SECRET=${JWT_SECRET}

# Expose the port the server runs on
EXPOSE 8080

# Set the entrypoint
CMD ["ethereum-fetcher"]
