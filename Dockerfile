FROM rust:slim-bookworm AS builder
WORKDIR /app

COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/monitoring-probe /usr/local/bin/monitoring-probe

ENV PORT=8080

EXPOSE 8080

CMD ["monitoring-probe"]
