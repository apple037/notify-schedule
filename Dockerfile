FROM rust:1.75.0-slim as build
LABEL name="notify-scedule"
LABEL version="0.1.0"
LABEL maintainer="Jasper the fan"

WORKDIR /usr/src/rust-axum
COPY . .

RUN cargo build --release

CMD [".target/release/notify-scedule"]
