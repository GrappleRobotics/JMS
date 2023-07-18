# To be invoked in project root
FROM rust:alpine

WORKDIR /work

# Needed to speed up build times, since having it in the project directory really slows down
# when mounted as a dev volume (especially on windows)
ENV CARGO_TARGET_DIR=/tmp/cargo

RUN apk add zlib-dev bash musl-dev g++ make libressl-dev

ADD jms-base /work/jms-base
RUN cd /work/jms-base && cargo build --release
