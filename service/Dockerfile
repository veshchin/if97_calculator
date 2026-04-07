FROM rust:1.88-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY if97_app_api ./if97_app_api
COPY service ./service

RUN cargo build --release -p if97_calculator_service

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/if97_calculator_service /usr/local/bin/if97_calculator_service

EXPOSE 8080
ENV IF97_SERVICE_ADDR=0.0.0.0:8080

ENTRYPOINT ["/usr/local/bin/if97_calculator_service"]
