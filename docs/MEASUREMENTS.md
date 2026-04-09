# Протокол измерений производительности

Цель: зафиксировать, что именно измеряется и какими командами, чтобы метрики были сравнимыми и воспроизводимыми.

## Термины и метрики

- `rows/s` (CPU): throughput бинарника `if97_loadtest`.  
  1 `row` = 1 строка из контрольного CSV (`core/tests/if97_rust_test_data.csv`). На каждую строку выполняются 4 вызова ядра: `pt/px`, `rhot`, `ph`, `ps`.
- `points/s` (gRPC): throughput бинарника `grpc_loadtest`.  
  1 `point` = 1 элемент SoA-массива внутри батча (`p[i], t[i]`) для endpoint `If97Service/CalculatePt`.

Важно: `rows/s` и `points/s` нельзя сравнивать напрямую. Это разные workload и разное количество вычислений на единицу.

## Профиль сборки

Производительность критично зависит от профиля сборки.

- для производительных замеров используйте `--release`
- для функциональных проверок достаточно `debug`

Бинарники `if97_loadtest` и `grpc_loadtest` печатают `profile=debug|release` в строке результата.

В `TESTS.md` бенчмарки обычно запускаются в `debug` (быстрее пересобирать); для "финальных" чисел используйте `--release`.

## Команды

### CPU: if97_loadtest (rows/s)

Сборка:

```bash
cargo build -p if97_calculator_service --bin if97_loadtest --features loadtest --release
```

Замер (пример):

```bash
./target/release/if97_loadtest --iters 200 --threads 8
```

### gRPC: grpc_loadtest (points/s)

Сборка:

```bash
cargo build -p if97_calculator_service --bin if97_calculator_service --release
cargo build -p if97_calculator_service --bin grpc_loadtest --release
```

Запуск сервиса (пример):

```bash
IF97_GRPC_ADDR=127.0.0.1:50051 \
IF97_KERNEL=core \
IF97_IO_THREADS=2 \
IF97_CPU_THREADS=8 \
IF97_MAX_IN_FLIGHT_PER_CONN=8 \
IF97_MAX_IN_FLIGHT_GLOBAL=8 \
IF97_DRAIN_SECS=2 \
RUST_LOG=info \
./target/release/if97_calculator_service
```

Замер транспорта (пример):

```bash
./target/release/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 500 --batch-size 2048 --in-flight 8
```

### Health API (grpc.health.v1.Health)

```bash
cargo run -p if97_calculator_service --bin grpc_healthcheck -- --addr 127.0.0.1:50051
```

## Где смотреть результаты

- актуальные логи прогонов и сводки: `TESTS.md`
- генерация отчета: `bash scripts/regenerate_tests_md.sh`
