FROM rust:alpine

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev libsecp256k1-dev

COPY . .

RUN rustup component add rustfmt

ENV RUSTFLAGS "-C target-feature=-crt-static"

RUN cargo build --tests

ENTRYPOINT [ "sh","-c","while true; do echo 'Running'; sleep 10; done" ]


