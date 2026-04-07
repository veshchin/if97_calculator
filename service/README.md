# IF97 Service

Отдельный HTTP-сервис над вычислительным ядром `if97_core`.

## Что реализовано

- `GET /healthz`
- `POST /api/v1/calculate/single`
- `POST /api/v1/calculate/table`
- `POST /api/v1/plot/dome`

Контракты запросов и ответов повторяют структуры из `if97_app_api`, поэтому десктопный, web и серверный слой используют один и тот же DTO-набор.

## Локальный запуск

```bash
cargo run -p if97_calculator_service
```

По умолчанию сервис слушает `0.0.0.0:8080`. Адрес можно переопределить через `IF97_SERVICE_ADDR`.

## Docker

```bash
docker build -t if97-calculator-service:0.1.4 -f service/Dockerfile .
docker run --rm -p 8080:8080 if97-calculator-service:0.1.4
```

## Kubernetes

Манифесты находятся в `service/k8s/`:

- `deployment.yaml` — основной Deployment с readiness/liveness probes
- `service.yaml` — ClusterIP-сервис
- `hpa.yaml` — HorizontalPodAutoscaler по CPU и памяти

Пример применения:

```bash
kubectl apply -f service/k8s/
```
