FROM rust as build

RUN USER=root cargo new --bin brane-api
WORKDIR /brane-api

# Copy over project manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This will prepare dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over project source code
COPY ./src ./src

# This will build a release binary
RUN rm ./target/release/deps/brane_api*
RUN cargo build --release

# Define final image
FROM ubuntu:20.04

RUN apt-get update && apt-get install -y \
    libsqlite3-dev \
    sqlite3 \
 && rm -rf /var/lib/apt/lists/*

# Copy the release binary from the build stage
COPY --from=build /brane-api/target/release/brane-api .

CMD ["./brane-api"]
