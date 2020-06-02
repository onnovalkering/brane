FROM rust as build

RUN apt-get update && apt-get install -y \
    cmake \
 && rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin brane-loop
WORKDIR /brane-loop

# Copy over project manifests
COPY ./Cargo.toml ./Cargo.toml

# This will prepare dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over project source code
COPY ./src ./src

# This will build a release binary
RUN rm ./target/release/deps/brane_loop*
RUN cargo build --release

# Define final image
FROM ubuntu:20.04

RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev\
 && rm -rf /var/lib/apt/lists/*

# Copy the release binary from the build stage
COPY --from=build /brane-loop/target/release/brane-loop .

CMD ["./brane-loop"]


