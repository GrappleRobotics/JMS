# Alpine images have trouble with openssl-sys crate, which is required for SSH, so we use
# debian images instead.
FROM rust:slim-buster AS chef
WORKDIR /app
RUN apt-get update && apt-get install --yes build-essential libssl-dev libssh2-1-dev pkg-config ca-certificates
RUN cargo install cargo-chef --locked

# Prepare Recipe
FROM chef AS planner
ADD . .
RUN cargo chef prepare --recipe-path recipe.json

# Build Application
FROM chef AS builder
# https://github.com/sfackler/rust-openssl/issues/1462
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ADD . .

RUN cargo build --release
RUN cargo run --bin jms-websocket --release -- gen-schema -f schema.json

# Create JMS runtime environment
FROM debian:buster-slim as rust_runtime
RUN apt-get update && apt-get install --yes libssl-dev ca-certificates && apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/jms-* /usr/local/bin
# The UI image is built off this image, and needs access to the schema, so we copy it into the final image. 
COPY --from=builder /app/schema.json /jms/schema.json

COPY docker-entrypoint.sh /usr/local/bin/
ENTRYPOINT ["docker-entrypoint.sh"]