# IF97 Calculator

`if97_calculator` — монорепозиторий с вычислительным ядром IAPWS-IF97 и несколькими способами доставки одного и того же функционала: desktop UI, web/Tauri UI и gRPC-сервис.

Репозиторий: <https://github.com/veshchin/if97_calculator>

## Документация (Rustdoc)

Опубликованные rustdoc-страницы (ядро + gRPC service + общие DTO):

- <https://veshchin.github.io/if97_calculator/>

Локальная сборка документации:

```bash
cargo doc -p if97_core -p if97_app_api -p if97_calculator_service --no-deps --open
```

## Демо и воспроизводимость

- протокол замеров: `docs/MEASUREMENTS.md`
- трассировка «задача -> модуль -> тест -> артефакт»: `docs/TRACEABILITY.md`
- preflight-проверка окружения: `bash scripts/preflight.sh`

## Состав проекта

- `core/` — вычислительное ядро `if97_core`
- `if97_app_api/` — общие DTO и контракты между UI и backend
- `wasm/` — интерфейс Yew + Tauri
- `fltk/` — нативный desktop-клиент на FLTK
- `service/` — gRPC-обёртка над ядром для Docker/Kubernetes

## Что умеет проект

- расчеты по входным парам `p-T`, `p-h`, `p-s`, `p-x`, `rho-T`
- табличный расчет с автопересчетом, импортом/экспортом и настройкой формата вывода
- сохранение точек и таблиц для последующей отрисовки
- построение диаграмм по выбранным осям (в т.ч. `p-T`, `p-v`, `T-s`, `h-s`), логарифмические шкалы, автомасштаб и ручные пределы
- системный журнал с экспортом в `.log`
- серверный режим для горизонтального масштабирования вычислений

## Быстрый старт

Требования:

- Rust toolchain через `rustup` (рекомендуемая версия зафиксирована в `rust-toolchain.toml`)
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

### Тесты DTO/сервиса/UI-backend

```bash
cargo test -p if97_app_api
cargo test -p if97_calculator_service
cargo test -p if97_calculator_native
```

Отчет с полными логами команд (тесты + бенчи) генерируется так:

```bash
bash scripts/regenerate_tests_md.sh
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

### Web (WASM/Yew в браузере)

```bash
cd wasm
trunk serve --open
```

### gRPC-сервис

```bash
IF97_GRPC_ADDR=0.0.0.0:50051 cargo run -p if97_calculator_service --bin if97_calculator_service
```

API описан в `service/proto/if97.proto`:

- `If97Service/CalculatePt` (bidirectional streaming, SoA: `p[]`, `t[]` -> `h[]`, `status[]`)
- gRPC health (`grpc.health.v1.Health`)

## Docker и Kubernetes

Сборка контейнера:

```bash
docker build -t if97-calculator-service:1.1.0 -f service/Dockerfile .
```

Запуск контейнера:

```bash
docker run --rm -p 50051:50051 if97-calculator-service:1.1.0
```

Kubernetes-манифесты лежат в `service/k8s/` и включают:

- `Deployment`
- `Service`
- `HorizontalPodAutoscaler`
- `PodDisruptionBudget`

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

Он проверяет workspace, собирает Tauri-приложение, FLTK-бинарники и публикует артефакты по тегу формата `vMAJOR.MINOR.PATCH` (например, `v1.1.0`).
