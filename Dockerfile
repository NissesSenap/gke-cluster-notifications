FROM rust:1.66 as build

RUN cargo new --bin app
WORKDIR /app

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
    && rm src/*.rs ./target/release/deps/gke*cluster*notifications*

COPY ./src ./src

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/cache/apt/* /var/lib/apt/lists/*

COPY --from=build /app/target/release/gke-cluster-notifications /app/
CMD ["/app/gke-cluster-notifications"]
