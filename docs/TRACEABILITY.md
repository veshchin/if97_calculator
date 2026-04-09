# Трассировка: задача -> модуль -> тест -> артефакт

Таблица предназначена для демонстрации: что именно проверяется, где лежит реализация, каким тестом это покрыто и какой артефакт можно приложить.

| Задача/требование | Модуль(и) | Тест(ы) | Артефакт |
|---|---|---|---|
| Корректность расчетов IF97 | `core/` | `cargo test -p if97_core` | Логи тестов, [`TESTS.md`](../TESTS.md) |
| Корректность DTO/serde | `if97_app_api/` | `cargo test -p if97_app_api` | Логи тестов, [`TESTS.md`](../TESTS.md) |
| Валидация gRPC батчей (happy-path + ошибки формата) | `service/` | `cargo test -p if97_calculator_service` | Логи тестов |
| Прямая проверка Health API | `service/` | `grpc_healthcheck` (бинарник) и `cargo test -p if97_calculator_service` | Вывод `grpc_healthcheck` |
| Smoke-проверка backend команд Tauri (без GUI) | `wasm/src-tauri/` | `cargo test -p if97_calculator_native` | Логи тестов |
| Производительность CPU (ядро) | `service/src/bin/if97_loadtest.rs` | запуск бенча по протоколу | [`docs/MEASUREMENTS.md`](MEASUREMENTS.md), [`TESTS.md`](../TESTS.md) |
| Производительность gRPC транспорта | `service/src/bin/grpc_loadtest.rs` | запуск бенча по протоколу | [`docs/MEASUREMENTS.md`](MEASUREMENTS.md), [`TESTS.md`](../TESTS.md) |
