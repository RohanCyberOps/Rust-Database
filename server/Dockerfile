# Use the official Rust image from Docker Hub
FROM rust:latest as builder

# Set a working directory inside the container
WORKDIR /usr/src/server

# Copy the project files into the container
COPY . .

# Build the Rust project
RUN cargo build --release

# Start a new stage for the final image
FROM debian:bookworm-slim

# Set a working directory inside the container
WORKDIR /usr/src/server

# Copy the built binary from the previous stage
COPY --from=builder /usr/src/server/target/release/server .

# Expose any necessary ports
EXPOSE 3000

# Command to run the application
CMD ["./server"]
