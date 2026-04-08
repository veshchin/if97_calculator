# `if97_calculator`: описание проекта

## 1. Назначение и результат

`if97_calculator` — монорепозиторий на Rust, который реализует вычисление теплофизических свойств воды и водяного пара по стандарту **IAPWS-IF97** и предоставляет один и тот же функционал через несколько “каналов доставки”:

- библиотечное ядро (`if97_core`) с публичным API;
- нативное desktop-приложение на FLTK;
- кроссплатформенный UI на базе Yew (WebAssembly) + Tauri (desktop);
- gRPC-микросервис для пакетных расчетов и интеграции в инфраструктуру (Docker/Kubernetes).

Проект ориентирован на “инженерный” сценарий: много запросов, табличные расчеты, графики/диаграммы, экспорт результатов, воспроизводимая сборка и автоматическая публикация артефактов.

## 2. Состав репозитория (workspaces/crates)

Rust workspace описан в `Cargo.toml` (корень репозитория). В состав входят:

- `core/` (`if97_core`) — вычислительное ядро IAPWS-IF97.
- `if97_app_api/` (`if97_app_api`) — общие DTO и контракты между UI и backend (сериализуемые структуры/enum’ы).
- `fltk/` (`if97_calculator_fltk`) — нативный desktop-клиент на FLTK.
- `wasm/` (`if97_calculator_web`) — web-frontend на Yew (WASM).
- `wasm/src-tauri/` (`if97_calculator_native`) — Tauri-backend (desktop) с командами `tauri::command`.
- `service/` (`if97_calculator_service`) — gRPC-сервис над ядром.

Также в репозитории присутствуют:

- `.github/workflows/docs.yml` — сборка rustdoc и публикация на GitHub Pages.
- `.github/workflows/release.yml` — релизный workflow по тегам `vMAJOR.MINOR.PATCH`.
- `service/Dockerfile`, `service/k8s/*.yaml` — контейнеризация и Kubernetes-манифесты.

## 3. Технологический стек

- Язык/платформа: Rust (workspace), WASM для web-части.
- UI:
  - FLTK (нативный desktop UI).
  - Yew (CSR) + Trunk (сборка WASM).
  - Tauri v2 (desktop упаковка и доступ к системным функциям через команды).
- Визуализация: `plotters` (в FLTK и web-части).
- Сервис: gRPC (tonic), health-check (tonic-health), async I/O (tokio), CPU-параллелизм (rayon).
- CI/CD: GitHub Actions; артефакты релиза собираются по тегам.

## 4. Архитектура и потоки данных

Ключевая идея архитектуры: **одно ядро** (`if97_core`) и несколько thin-layer “адаптеров” поверх него.

Потоки данных:

- FLTK UI:
  - пользовательский ввод (строки/таблица) → парсинг → вызов `if97_core::If97::*` → отображение результата → (опционально) сохранение/экспорт.
- Yew frontend (WASM) + Tauri:
  - UI-компоненты формируют запросы в DTO (`if97_app_api`) → вызов команд Tauri (`invoke`) → вычисления выполняются в Tauri-backend через `if97_core` → UI получает результат как DTO и отображает его.
- Tauri:
  - web-frontend вызывает `window.__TAURI__.core.invoke(...)` → Tauri-backend выполняет расчет через `if97_core` → возвращает сериализуемый DTO.
- gRPC service:
  - клиент отправляет поток батчей `PtBatchRequest` → сервис валидирует формат батча → отправляет вычисления в CPU-пул rayon → возвращает `PtBatchResponse` в поток по мере готовности.

## 5. Ядро `if97_core` (IAPWS-IF97)

### 5.1. Публичный API

Публичный фасад — `if97_core::If97`:

- `If97::pt(p, t)` — прямой расчет по давлению и температуре.
- `If97::ph(p, h)` — обратный расчет по давлению и энтальпии.
- `If97::ps(p, s)` — обратный расчет по давлению и энтропии.
- `If97::px(p, x)` — расчет в двухфазной области (линия насыщения и смесь).
- `If97::rhot(rho, t)` — расчет по плотности и температуре (поиск давления под заданную плотность).
- `If97::metastable_pt(p, t)` — метастабильный пар (расширение Region 2 с ограничением по давлению).

Результат вычислений — агрегированная структура `WaterState` (p, T, v, rho, h, s, u, cp, w, region).

### 5.2. Слои ядра

Внутренняя архитектура ядра разделена на слои:

- `domain` — типы предметной области (единицы измерения, ошибки, состояние).
- `engine` — маршрутизация: валидация входа, классификация региона, выбор “плана” вычисления и запуск workflow.
- `regions` — реализация уравнений регионов IF97 (1–5) и решатели для обратных расчетов.
- `topology` — фазовые границы и работа с линией насыщения (Region 4), классификация областей (включая границу B23).
- `calculator` — фасад `If97`, который переводит входы в `engine::CalculationRequest`.

### 5.3. Типобезопасность единиц

В публичном API используются newtype’ы единиц (`MegaPascal`, `Kelvin`, и т.п.), чтобы:

- уменьшить риск перепутать размерности;
- упростить чтение интерфейса (очевидно, в каких единицах подаются значения).

### 5.4. Кодогенерация таблиц коэффициентов (build.rs)

`core/build.rs` на этапе компиляции читает CSV-таблицы коэффициентов из `core/data/` и генерирует статический Rust-код (массивы констант).

Зачем это сделано:

- таблицы становятся частью бинарника (нет runtime I/O);
- доступ к коэффициентам “zero-cost” (простые срезы констант);
- сборка детерминирована (файлы сортируются перед генерацией).

Сгенерированный код подключается модулем `core/src/tables.rs` через `include!(concat!(env!("OUT_DIR"), "/tables.rs"))`.

### 5.5. Двухфазная область и купол насыщения

Ядро поддерживает расчет в двухфазной области через `px(p, x)` и отдельный модуль `saturation`.

В UI для построения диаграмм используется купол насыщения (границы `x=0` и `x=1`), который формируется адаптивным уточнением (subdivision) по параметру температуры:

- делается предварительная оценка “размахов” по осям диаграммы;
- далее сегменты уточняются рекурсивно до тех пор, пока отклонение от линейной аппроксимации не станет достаточно малым;
- для оси удельного объема `v` используется метрика `ln(v)`, чтобы получить плотность точек в области малого `v`.

Реализации купола есть в двух местах:

- FLTK: `fltk/src/plot/renderer.rs` (рендер графика/купола в файл для экспорта).
- Tauri backend: `wasm/src-tauri/src/lib.rs` (команда `calculate_dome` для UI).

### 5.6. Валидация и ошибки

В `engine/validator.rs` проверяются диапазоны:

- давление `p` в границах IF97;
- температура `T` в допустимом диапазоне;
- плотность `rho > 0`;
- степень сухости `x` в `[0; 1]`;
- метастабильный пар ограничен `p <= 10 МПа`.

Ошибки возвращаются как `If97Error` (например, `OutOfBounds`, `PhaseBoundaryError`, `InvalidInput`).

### 5.7. Тестирование ядра

В `core/tests/` есть интеграционные тесты и CSV-набор контрольных точек:

- unit-тесты проверяют граничные кривые, классификацию регионов, обратные решатели и таблицы верификации;
- интеграционный тест `integration_csv_test.rs` сверяет расчет по набору точек из CSV.

Запуск:

```bash
cargo test -p if97_core
```

## 6. `if97_app_api`: общие DTO и контракты

`if97_app_api` содержит сериализуемые структуры и enum’ы, которые используются между UI и backend:

- `InputMode` — режим ввода (pt/ph/ps/px/rhot).
- `DiagramKind` — тип диаграммы (Pt, Pv, Ts, Hs, и т.п.).
- `SingleCalcRequest`, `TableCalcRequest`, `TableRowResult` — запросы/ответы для UI.
- `StateDto` — сериализуемое состояние (числа + строка региона).
- `LogEntryDto` — записи логов для UI.

Особенность: JSON не поддерживает `NaN` и `±Infinity`, поэтому `StateDto` кодирует такие значения как строки `"nan"`, `"inf"`, `"-inf"` (и допускает `null` как `+Infinity`).

## 7. Desktop UI (FLTK)

Крейт: `fltk/` (`if97_calculator_fltk`).

Основные возможности:

- одиночные расчеты по выбранной входной паре (`InputMode`);
- табличный расчет (вставка/загрузка текстовой таблицы, обработка ошибок построчно);
- построение диаграмм и отображение купола насыщения;
- сохранение точек/таблиц как “dataset” и управление видимостью на графике;
- экспорт логов и артефактов (например, графика) в файлы.

Архитектура:

- `fltk/src/state.rs` — `AppState` и типы сообщений/данных (точки, таблицы, datasets).
- `fltk/src/ui/*` — вкладки UI (single/batch/plot/about).
- `fltk/src/plot/renderer.rs` — подготовка данных и отрисовка графиков.

Запуск:

```bash
cargo run -p if97_calculator_fltk --release
```

## 8. Web UI (Yew/WASM) + Tauri (desktop)

### 8.1. Web-frontend (Yew)

Крейт: `wasm/` (`if97_calculator_web`).

Содержит:

- UI-компоненты `wasm/src/ui/*` (single/table/plots/about+logs);
- обертки над Tauri invoke API: `wasm/src/tauri_api.rs`;
- типы/логгер/отрисовка.

Сборка frontend (артефакт в `wasm/dist/`):

```bash
cd wasm
trunk build --release
```

Примечание: текущая реализация frontend’а использует Tauri `invoke` для вычислений и системных действий, поэтому для полноценной работы расчетов требуется запуск внутри Tauri.

Запуск desktop-версии (dev) делается через Tauri:

```bash
cd wasm
cargo tauri dev
```

### 8.2. Tauri backend

Крейт: `wasm/src-tauri/` (`if97_calculator_native`).

Реализует набор `#[tauri::command]`:

- `calculate_single` — одиночный расчет, возвращает `StateDto`;
- `calculate_table` — табличный расчет, возвращает список `TableRowResult`;
- `calculate_dome` — купол насыщения (точки) для выбранной диаграммы;
- `load_file_dialog`, `save_file_dialog`, `save_plot_dialog` — диалоги файлов;
- `read_logs`, `clear_logs` — работа с буфером логов backend’а.

Важная деталь: тяжелые вычисления выполняются через `tauri::async_runtime::spawn_blocking`, чтобы не блокировать async runtime.

Запуск Tauri (dev):

```bash
cd wasm
cargo tauri dev
```

## 9. gRPC-сервис (`service/`)

### 9.1. API и формат данных

Протокол описан в `service/proto/if97.proto`.

Сервис предоставляет метод:

- `If97Service/CalculatePt` — bidirectional streaming.

Формат данных: **Struct-of-Arrays (SoA)**.

- вход: `PtBatchRequest { batch_id, p[], t[] }`
- выход: `PtBatchResponse { batch_id, h[], status[], batch_error_* }`

Смысл:

- `h[i]` — энтальпия для пары `(p[i], t[i])`;
- `status[i]` — код статуса (0 = OK, иначе ошибка ядра);
- `batch_error_code/message` — ошибки валидации “на уровне батча” (например, несовпадение длин массивов).

### 9.2. Модель параллелизма и backpressure

Сервис разделяет I/O и CPU:

- I/O: `tokio` runtime (несколько worker threads).
- CPU: глобальный `rayon` пул фиксированного размера.

Backpressure:

- per-connection лимит in-flight батчей (`IF97_MAX_IN_FLIGHT_PER_CONN`);
- global лимит in-flight батчей на процесс (`IF97_MAX_IN_FLIGHT_GLOBAL`).

Вычисления запускаются в `rayon::spawn`, а результаты отправляются обратно в gRPC stream через `blocking_send`.

### 9.3. Конфигурация (env)

Ключевые переменные окружения (см. `service/src/config.rs`):

- `IF97_GRPC_ADDR` (default `0.0.0.0:50051`)
- `IF97_IO_THREADS`
- `IF97_CPU_THREADS` (по умолчанию определяется по физическим ядрам с учетом cgroup quota/cpuset)
- `IF97_MAX_IN_FLIGHT_PER_CONN`
- `IF97_MAX_IN_FLIGHT_GLOBAL`
- `IF97_MAX_BATCH_LEN` (default `1_000_000`)
- `IF97_GRPC_MAX_MSG_BYTES` (default `32 MiB`)
- `IF97_DRAIN_SECS` (default `2`)
- `IF97_KERNEL` (`core|stub`)

Локальный запуск:

```bash
IF97_GRPC_ADDR=0.0.0.0:50051 cargo run -p if97_calculator_service --bin if97_calculator_service
```

### 9.4. Health-check и завершение

Сервис поднимает `tonic-health` и выставляет статус `Serving/NotServing`.

При получении сигнала завершения (SIGINT/SIGTERM):

- уведомляет обработчики;
- выставляет health в `NotServing`;
- делает `drain` (пауза `IF97_DRAIN_SECS`) для корректного выключения под нагрузкой.

### 9.5. Docker и Kubernetes

- Dockerfile: `service/Dockerfile` (multi-stage, ставит `protobuf-compiler` в builder-стадии).
- Kubernetes: `service/k8s/*` (Deployment/Service/HPA/PDB/Job для loadtest).

## 10. Сборка и CI/CD

### 10.1. Локальные команды (минимум)

Проверка workspace:

```bash
cargo check --workspace
```

Проверка web-таргета:

```bash
cargo check -p if97_calculator_web --target wasm32-unknown-unknown
```

Сборка документации (rustdoc):

```bash
cargo doc -p if97_core -p if97_app_api -p if97_calculator_service --no-deps
```

### 10.2. GitHub Actions

Документация (Pages):

- workflow: `.github/workflows/docs.yml`
- собирает rustdoc для core/api/service и публикует на GitHub Pages.

Релиз:

- workflow: `.github/workflows/release.yml`
- триггер: push тега формата `vMAJOR.MINOR.PATCH` (pattern `v*.*.*`)
- jobs:
  - `validate` (Ubuntu): `cargo check --workspace`, web check, `cargo test -p if97_core`, ставит системные зависимости + `protobuf-compiler`
  - `release` (Windows/macOS/Linux): собирает FLTK и Tauri bundles и публикует в GitHub Release
  - `docs-and-web` (Ubuntu): собирает `rustdoc` и web (`trunk build --release`), загружает как assets в релиз

### 10.3. Версионирование и теги

Теги релиза: только `vX.Y.Z` (SemVer).

Это упрощает:

- предсказуемые релизы (только корректные теги триггерят сборки);
- соответствие требованиям GitHub tooling и экосистемы.

## 11. Примечания для описания в дипломе

См. отдельный файл [`THESIS_OUTLINE.md`](THESIS_OUTLINE.md).
