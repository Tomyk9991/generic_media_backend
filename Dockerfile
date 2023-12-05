# Use a Rust base image with Debian testing
FROM rust:1.71 AS chef

RUN cargo install cargo-chef
# Set the working directory
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the Project
COPY . .
#Build the actual project
RUN cargo build --release

# Create a new stage for the final image
FROM debian:testing-slim

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the previous stage
COPY --from=build /app/target/release/image_server_backend .

# Copy the .env file into the image
COPY .env .env
COPY changelog ./changelog

# Expose port 80
EXPOSE 80

# Start the application with environment variables from .env
CMD ["./image_server_backend"]




#################### OLD ####################
# Use a Rust base image with Debian testing
#FROM rust:1.71 AS build
#
## Set the working directory
#WORKDIR /app
#
## Copy the Cargo.toml and Cargo.lock files to cache dependencies
#COPY Cargo.toml Cargo.lock ./
#
## Copy the rest of the source code
#COPY src ./src
#
## Build the actual project
#RUN cargo build --release
#
## Create a new stage for the final image
#FROM debian:testing-slim
#
## Set the working directory
#WORKDIR /app
#
## Copy the compiled binary from the previous stage
#COPY --from=build /app/target/release/image_server_backend .
#
## Copy the .env file into the image
#COPY .env .env
#COPY changelog ./changelog
#
## Expose port 80
#EXPOSE 80
#
## Start the application with environment variables from .env
#CMD ["./image_server_backend"]