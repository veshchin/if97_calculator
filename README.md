# IF97 Calculator

`if97_calculator` — монорепозиторий с вычислительным ядром IAPWS-IF97 и несколькими способами доставки одного и того же функционала: desktop UI, web/Tauri UI и HTTP-сервис.

Репозиторий: <https://github.com/veshchin/if97_calculator>

## Состав проекта

- `core/` — вычислительное ядро `if97_core`
- `if97_app_api/` — общие DTO и контракты между UI и backend
- `wasm/` — интерфейс Yew + Tauri
- `fltk/` — нативный desktop-клиент на FLTK
- `service/` — HTTP-обёртка над ядром для Docker/Kubernetes

## Что умеет проект

- расчеты по входным парам `p-T`, `p-h`, `p-s`, `p-x`, `rho-T`
- табличный расчет с автопересчетом, импортом и экспортом
- сохранение точек и таблиц для последующей отрисовки
- построение диаграмм `pT`, `pV`, `pS`, `pH`, `TV`, `TS`, `TH`, `HS`
- системный журнал с экспортом в `.log`
- серверный режим для горизонтального масштабирования вычислений

## Быстрый старт

Требования:

- Rust toolchain через `rustup`
- для Tauri: системные зависимости Tauri и `cargo-tauri`
- для web frontend: `trunk`
- для контейнерного режима: Docker

### Проверка workspace

```bash
cargo check --workspace
```

### Тесты ядра

```bash
cargo test -p if97_core
```

### FLTK-приложение

```bash
cargo run -p if97_calculator_fltk --release
```

### Tauri-приложение

```bash
cd wasm
cargo tauri dev
```

### HTTP-сервис

```bash
cargo run -p if97_calculator_service
```

Сервис поднимается на `0.0.0.0:8080` и предоставляет:

- `GET /healthz`
- `POST /api/v1/calculate/single`
- `POST /api/v1/calculate/table`
- `POST /api/v1/plot/dome`

## Docker и Kubernetes

Сборка контейнера:

```bash
docker build -t if97-calculator-service:0.1.4 -f service/Dockerfile .
```

Запуск контейнера:

```bash
docker run --rm -p 8080:8080 if97-calculator-service:0.1.4
```

Kubernetes-манифесты лежат в `service/k8s/` и включают:

- `Deployment`
- `Service`
- `HorizontalPodAutoscaler`

Применение:

```bash
kubectl apply -f service/k8s/
```

## Архитектура ядра

Актуальный внутренний разрез `core/`:

- `domain` — типы предметной области и ошибки
- `engine` — маршрутизация, валидация и workflow
- `regions` — региональные модели стандарта
- `topology` — фазовые границы и классификация
- `calculator` — стабильный публичный фасад `If97`

Это позволяет рефакторить внутреннюю архитектуру без переписывания математики и решателей.

## Релизы

Релизный workflow находится в `.github/workflows/release.yml`.

Он проверяет workspace, собирает Tauri-приложение, FLTK-бинарники и публикует артефакты по тегу формата `v*`.
