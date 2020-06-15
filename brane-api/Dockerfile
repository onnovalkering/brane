FROM rust as build

RUN apt-get update && apt-get install -y \
    cmake \
 && rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin brane-api
WORKDIR /brane-api

# Copy over project manifests
COPY ./Cargo.toml ./Cargo.toml

# This will prepare dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over project source code
COPY ./migrations ./migrations
COPY ./src ./src
COPY ./diesel.toml ./diesel.toml

# This will build a release binary
RUN rm ./target/release/deps/brane_api*
RUN cargo build --release

# Define final image
FROM ubuntu:20.04

RUN apt-get update && apt-get install -y \
    gnupg2 \
    libpq-dev \
    libssl-dev \
    wget \
 && rm -rf /var/lib/apt/lists/*

RUN . /etc/os-release \
 && sh -c "echo 'deb http://download.opensuse.org/repositories/devel:/kubic:/libcontainers:/stable/x${NAME}_${VERSION_ID}/ /' > /etc/apt/sources.list.d/devel:kubic:libcontainers:stable.list" \
 && wget -nv https://download.opensuse.org/repositories/devel:kubic:libcontainers:stable/x${NAME}_${VERSION_ID}/Release.key -O- | apt-key add - \
 && apt-get update \
 && apt-get install -y skopeo \
 && rm -rf /var/lib/apt/lists/*

# Copy the release binary from the build stage
COPY --from=build /brane-api/target/release/brane-api .

CMD ["./brane-api"]
