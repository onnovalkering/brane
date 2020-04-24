FROM rust as build

RUN USER=root cargo new --bin executor
WORKDIR /executor

# Copy over project manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This will prepare dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over project source code
COPY ./src ./src

# This will build a release binary
RUN rm ./target/release/deps/executor*
RUN cargo build --release

# Define final image
FROM rust

# Copy the release binary from the build stage
COPY --from=build /executor/target/release/executor .

CMD ["./executor"]