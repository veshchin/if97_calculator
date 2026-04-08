FROM rust:1.88-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY if97_app_api ./if97_app_api
COPY fltk ./fltk
COPY wasm ./wasm
COPY service ./service

RUN apt-get update \
    && apt-get install -y --no-install-recommends protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release -p if97_calculator_service --bin if97_calculator_service --bin grpc_loadtest

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -u 10001 -m app

WORKDIR /app
COPY --from=builder /app/target/release/if97_calculator_service /usr/local/bin/if97_calculator_service
COPY --from=builder /app/target/release/grpc_loadtest /usr/local/bin/grpc_loadtest

EXPOSE 50051
ENV IF97_GRPC_ADDR=0.0.0.0:50051
ENV RUST_LOG=info
USER 10001:10001

ENTRYPOINT ["/usr/local/bin/if97_calculator_service"]
