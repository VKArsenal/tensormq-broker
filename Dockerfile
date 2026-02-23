FROM rust:1.93.1-bookworm AS builder

WORKDIR /usr/src/tensormq-broker

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/tensormq-broker/target/release/tensormq-broker /usr/local/bin/tensormq-broker

EXPOSE 59321

CMD ["tensormq-broker"]