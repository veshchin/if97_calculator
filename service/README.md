# IF97 Service

Отдельный gRPC-сервис над вычислительным ядром `if97_core`.

## Что реализовано

- gRPC `If97Service/CalculatePt` (bidirectional streaming, SoA: `p[]`, `t[]` -> `h[]`, `status[]`)
- gRPC health (`grpc.health.v1.Health`)

Сетевой ввод-вывод работает в `tokio` runtime, вычисления выполняются в фиксированном `rayon` pool (без `spawn_blocking`).

## Локальный запуск

```bash
IF97_GRPC_ADDR=0.0.0.0:50051 cargo run -p if97_calculator_service --bin if97_calculator_service
```

Основные переменные окружения:

- `IF97_GRPC_ADDR` (default `0.0.0.0:50051`)
- `IF97_KERNEL` (`core` | `stub`, default `core`)
- `IF97_IO_THREADS` (default `2`)
- `IF97_CPU_THREADS` (default = physical cores)
- `IF97_MAX_IN_FLIGHT_PER_CONN` (default = `IF97_CPU_THREADS`)
- `IF97_MAX_IN_FLIGHT_GLOBAL` (default = `IF97_CPU_THREADS`)
- `IF97_MAX_BATCH_LEN` (default `1000000`)
- `IF97_GRPC_MAX_MSG_BYTES` (default `33554432`)
- `IF97_DRAIN_SECS` (default `2`)

## Docker

```bash
docker build -t if97-calculator-service:0.1.4 -f service/Dockerfile .
docker run --rm -p 50051:50051 if97-calculator-service:0.1.4
```

Нагрузочный gRPC-тест внутри контейнера:

```bash
docker run --rm --entrypoint /usr/local/bin/grpc_loadtest if97-calculator-service:0.1.4 -- --addr 127.0.0.1:50051 --streams 1 --batches 2000 --batch-size 2048 --in-flight 8
```

## Kubernetes

Манифесты находятся в `service/k8s/`:

- `deployment.yaml` — основной Deployment (gRPC probes, securityContext, drain)
- `service.yaml` — ClusterIP-сервис
- `hpa.yaml` — HorizontalPodAutoscaler по CPU и памяти
- `pdb.yaml` — PodDisruptionBudget
- `loadtest-job.yaml` — Job для быстрого gRPC-loadtest внутри кластера

Пример применения:

```bash
kubectl apply -f service/k8s/
```
