# Прогоны тестов и бенчмарков

Дата отчета (локальное время): `2026-04-09T13:45:46+0300`  
Дата отчета (UTC): `2026-04-09T10:45:46Z`

Репозиторий: `/Users/mrk-3/Проекты/if97_calculator`  
Коммит: `c76dbafa2e2f93f72d2ceae4eadd7f7de78ac4c3`

Важно про неизменность файлов:
- Рабочее дерево репозитория было изменено перед прогоном (ниже приведен `git status --porcelain`).
- Это нормально для локального отчета: результаты соответствуют текущему checkout.

`git status --porcelain`:
```text
 M README.md
 M service/Cargo.toml
 M service/README.md
 M service/src/bin/grpc_loadtest.rs
 M service/src/bin/if97_loadtest.rs
 M service/src/grpc.rs
 M wasm/src-tauri/src/lib.rs
?? .github/workflows/ci.yml
?? TESTS.md
?? docs/
?? scripts/
?? service/src/bin/grpc_healthcheck.rs
```
- Все сборки/прогоны выполнялись в изолированной копии репозитория: `/tmp/if97_calculator_tests.33J7CY/repo`.
- Скрипт генерации отчета перезаписывает только этот `TESTS.md` (код проекта во время прогонов не модифицируется).

## Условия стенда

OS (uname):
```text
Darwin MacBook-Air-Mark.local 25.4.0 Darwin Kernel Version 25.4.0: Thu Mar 19 19:33:09 PDT 2026; root:xnu-12377.101.15~1/RELEASE_ARM64_T8112 arm64
```

macOS (sw_vers):
```text
ProductName:		macOS
ProductVersion:		26.4
BuildVersion:		25E246
```

Hardware (system_profiler, без серийников/UUID):
```text
Hardware:

    Hardware Overview:

      Model Name: MacBook Air
      Model Identifier: Mac14,2
      Model Number: MLY33LL/A
      Chip: Apple M2
      Total Number of Cores: 8 (4 Performance and 4 Efficiency)
      Memory: 8 GB
      System Firmware Version: 18000.101.7
      OS Loader Version: 18000.101.7
      Activation Lock Status: Enabled

```

Rust toolchain:
```text
rustc 1.94.0 (4a4ef493e 2026-03-02)
cargo 1.94.0 (85eff7c80 2026-01-15)
rustup 1.28.2 (2025-04-28)
```

Установленные targets:
```text
aarch64-apple-darwin
wasm32-unknown-unknown
x86_64-pc-windows-msvc
```

Сторонние инструменты:
```text
trunk 0.21.14
tauri-cli 2.10.1
```

Замечания по окружению/ограничениям:
- Если в окружении задан `NO_COLOR=1`, то `trunk build` может падать. Исправление: `NO_COLOR=true trunk ...`.
- В sandbox-режиме могут быть запрещены некоторые системные вызовы и локальный `listen` на сокетах; для FLTK (feature `fltk-bundled`) и запуска gRPC-сервера/клиента может потребоваться прогон вне песочницы.

## Прогоны тестов

Ниже приведены команды и полный вывод (stdout/stderr), пригодный для машинного парсинга.


### if97_app_api

Команда:
```bash
cargo test -p if97_app_api --frozen --color never -- --nocapture
```

Вывод:
```text
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling zmij v1.0.21
   Compiling serde_json v1.0.149
   Compiling itoa v1.0.18
   Compiling memchr v2.8.0
   Compiling syn v2.0.117
   Compiling serde_derive v1.0.228
   Compiling if97_app_api v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/if97_app_api)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.02s
     Running unittests src/lib.rs (target/debug/deps/if97_app_api-c4a7e495d2559c9b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/serde_nonfinite.rs (target/debug/deps/serde_nonfinite-01e1412675c6e29b)

running 2 tests
test state_dto_serializes_non_finite_as_strings ... ok
test state_dto_deserializes_non_finite_strings ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests if97_app_api

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Оценка результата:
- `OK`: тесты прошли.

### if97_core

Команда:
```bash
cargo test -p if97_core --frozen --color never -- --nocapture
```

Вывод:
```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling serde_core v1.0.228
   Compiling once_cell v1.21.4
   Compiling if97_core v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/core)
   Compiling memchr v2.8.0
   Compiling pin-project-lite v0.2.17
   Compiling ryu v1.0.23
   Compiling tracing-core v0.1.36
   Compiling csv-core v0.1.13
   Compiling syn v2.0.117
   Compiling csv v1.4.0
   Compiling tracing-attributes v0.1.31
   Compiling serde_derive v1.0.228
   Compiling tracing v0.1.44
   Compiling serde v1.0.228
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.38s
     Running unittests src/lib.rs (target/debug/deps/if97_core-026bdf7c6ee95e4f)

running 18 tests
test tests::tests::test_boundary_2bc ... ok
test tests::tests::test_determine_region ... ok
test tests::tests::test_region1_verification_table ... ok
test tests::tests::test_region1_backward_t_ps ... ok
test tests::tests::test_region2_backward_t_ph_official ... ok
test tests::tests::test_facade_ph_correct_regions ... ok
test tests::tests::test_continuity_boundary_1_3 ... ok
test tests::tests::test_region2_verification_table ... ok
test tests::tests::test_facade_ps_correct_regions ... ok
test tests::tests::test_region2_metastable_official ... ok
test tests::tests::test_region2_backward_t_ps_official ... ok
test tests::tests::test_region4_saturation_lines ... ok
test tests::tests::test_region3_backward_ph_solver ... ok
test tests::tests::test_region3_backward_ps_solver ... ok
test tests::tests::test_region5_verification_table ... ok
test tests::tests::test_region5_backward_iterative ... ok
test tests::tests::test_continuity_boundary_b23 ... ok
test tests::tests::test_region3_verification_table ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/integration_csv_test.rs (target/debug/deps/integration_csv_test-c651cea2b655dc0e)

running 1 test
Успешно пройдено точек (pT): 142
Успешно пройдено точек (rhoT): 142
Успешно пройдено точек (px): 100
Успешно пройдено точек (ph): 236
Успешно пройдено точек (ps): 236
test test_core_all_inputs_against_csv ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/load_stress_test.rs (target/debug/deps/load_stress_test-3ea53ec8787a7cff)

running 1 test
test load_stress_core_csv ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests if97_core

running 1 test
test core/src/lib.rs - (line 30) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.52s; merged doctests compilation took 0.18s
```

Оценка результата:
- `OK`: unit/integration/doctest прошли; ignored стресс-тест запускается отдельно как бенч.

### if97_calculator_service

Команда:
```bash
cargo test -p if97_calculator_service --frozen --color never -- --nocapture
```

Вывод:
```text
   Compiling libc v0.2.183
   Compiling bytes v1.11.1
   Compiling futures-core v0.3.32
   Compiling cfg-if v1.0.4
   Compiling hashbrown v0.16.1
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.102
   Compiling syn v2.0.117
   Compiling tracing-core v0.1.36
   Compiling slab v0.4.12
   Compiling either v1.15.0
   Compiling itertools v0.14.0
   Compiling http v1.4.0
   Compiling indexmap v2.13.0
   Compiling futures-task v0.3.32
   Compiling futures-util v0.3.32
   Compiling futures-sink v0.3.32
   Compiling tower-service v0.3.3
   Compiling zerocopy v0.8.47
   Compiling rustix v1.1.4
   Compiling http-body v1.0.1
   Compiling errno v0.3.14
   Compiling mio v1.2.0
   Compiling socket2 v0.6.3
   Compiling signal-hook-registry v1.4.8
   Compiling httparse v1.10.1
   Compiling getrandom v0.4.2
   Compiling getrandom v0.2.17
   Compiling crossbeam-utils v0.8.21
   Compiling rustversion v1.0.22
   Compiling prettyplease v0.2.37
   Compiling fnv v1.0.7
   Compiling autocfg v1.5.0
   Compiling try-lock v0.2.5
   Compiling atomic-waker v1.1.2
   Compiling smallvec v1.15.1
   Compiling tower-layer v0.3.3
   Compiling regex-syntax v0.8.10
   Compiling bitflags v2.11.0
   Compiling want v0.3.1
   Compiling indexmap v1.9.3
   Compiling rand_core v0.6.4
   Compiling futures-channel v0.3.32
   Compiling sync_wrapper v1.0.2
   Compiling fixedbitset v0.5.7
   Compiling httpdate v1.0.3
   Compiling pin-utils v0.1.0
   Compiling fastrand v2.3.0
   Compiling regex-automata v0.4.14
   Compiling petgraph v0.7.1
   Compiling tempfile v3.27.0
   Compiling http-body-util v0.1.3
   Compiling mime v0.3.17
   Compiling hashbrown v0.12.3
   Compiling log v0.4.29
   Compiling tokio-macros v2.6.1
   Compiling tracing-attributes v0.1.31
   Compiling prost-derive v0.13.5
   Compiling serde_derive v1.0.228
   Compiling async-trait v0.1.89
   Compiling tokio v1.50.0
   Compiling ppv-lite86 v0.2.21
   Compiling regex v1.12.3
   Compiling rand_chacha v0.3.1
   Compiling pin-project-internal v1.1.11
   Compiling tracing v0.1.44
   Compiling heck v0.5.0
   Compiling multimap v0.10.1
   Compiling axum-core v0.4.5
   Compiling rand v0.8.5
   Compiling prost v0.13.5
   Compiling async-stream-impl v0.3.6
   Compiling crossbeam-epoch v0.9.18
   Compiling tower v0.5.3
   Compiling percent-encoding v2.3.2
   Compiling pin-project v1.1.11
   Compiling matchit v0.7.3
   Compiling rayon-core v1.13.0
   Compiling prost-types v0.13.5
   Compiling memchr v2.8.0
   Compiling async-stream v0.3.6
   Compiling crossbeam-deque v0.8.6
   Compiling socket2 v0.5.10
   Compiling serde v1.0.228
   Compiling prost-build v0.13.5
   Compiling lazy_static v1.5.0
   Compiling base64 v0.22.1
   Compiling tracing-log v0.2.0
   Compiling sharded-slab v0.1.7
   Compiling tonic-build v0.12.3
   Compiling axum v0.7.9
   Compiling thread_local v1.1.9
   Compiling nu-ansi-term v0.50.3
   Compiling if97_core v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/core)
   Compiling num_cpus v1.17.0
   Compiling rayon v1.11.0
   Compiling if97_calculator_service v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/service)
   Compiling tokio-util v0.7.18
   Compiling tokio-stream v0.1.18
   Compiling h2 v0.4.13
   Compiling tower v0.4.13
   Compiling matchers v0.2.0
   Compiling tracing-subscriber v0.3.23
   Compiling hyper v1.8.1
   Compiling hyper-util v0.1.20
   Compiling hyper-timeout v0.5.2
   Compiling tonic v0.12.3
   Compiling tonic-health v0.12.3
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.18s
     Running unittests src/lib.rs (target/debug/deps/if97_calculator_service-02ed73526bb008a8)

running 5 tests
test grpc::tests::grpc_calculate_pt_valid_batch ... ok
test grpc::tests::grpc_calculate_pt_batch_too_large ... ok
test grpc::tests::grpc_calculate_pt_length_mismatch ... ok
test grpc::tests::grpc_healthcheck_serving ... ok
test grpc::tests::dispatcher_global_in_flight_blocks_second_dispatch ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running unittests src/bin/grpc_healthcheck.rs (target/debug/deps/grpc_healthcheck-0d4f4f5b6ff62fca)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/grpc_loadtest.rs (target/debug/deps/grpc_loadtest-9d3e0358f09a1ff2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/if97_calculator_service-1356682ef70f8b7c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests if97_calculator_service

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Оценка результата:
- `OK`: пакет собирается; имеются тесты на валидацию батчей и health/backpressure.

### if97_calculator_fltk (FLTK desktop)

Команда:
```bash
cargo test -p if97_calculator_fltk --frozen --color never -- --nocapture
```

Вывод:
```text
   Compiling stable_deref_trait v1.2.1
   Compiling core-foundation-sys v0.8.7
   Compiling litemap v0.8.1
   Compiling writeable v0.6.2
   Compiling bitflags v1.3.2
   Compiling crc32fast v1.5.0
   Compiling syn v2.0.117
   Compiling icu_properties_data v2.1.2
   Compiling icu_normalizer_data v2.1.1
   Compiling semver v1.0.27
   Compiling simd-adler32 v0.3.9
   Compiling num-traits v0.2.19
   Compiling shlex v1.3.0
   Compiling rustc_version v0.4.1
   Compiling find-msvc-tools v0.1.9
   Compiling adler2 v2.0.1
   Compiling smallvec v1.15.1
   Compiling cc v1.2.58
   Compiling miniz_oxide v0.8.9
   Compiling pathfinder_simd v0.5.5
   Compiling core-foundation v0.9.4
   Compiling foreign-types-shared v0.3.1
   Compiling fdeflate v0.3.7
   Compiling core-graphics-types v0.1.3
   Compiling option-ext v0.2.0
   Compiling byteorder v1.5.0
   Compiling flate2 v1.1.9
   Compiling cmake v0.1.58
   Compiling crossbeam-utils v0.8.21
   Compiling color_quant v1.1.0
   Compiling dirs-sys v0.5.0
   Compiling bytemuck v1.25.0
   Compiling png v0.17.16
   Compiling percent-encoding v2.3.2
   Compiling fltk-sys v1.5.22
   Compiling bitflags v2.11.0
   Compiling weezl v0.1.12
   Compiling plotters-backend v0.3.7
   Compiling font-kit v0.14.3
   Compiling jpeg-decoder v0.3.2
   Compiling utf8_iter v1.0.4
   Compiling same-file v1.0.6
   Compiling gif v0.12.0
   Compiling walkdir v2.5.0
   Compiling pathfinder_geometry v0.5.1
   Compiling form_urlencoded v1.2.2
   Compiling image v0.24.9
   Compiling dirs v6.0.0
   Compiling iana-time-zone v0.1.65
   Compiling float-ord v0.3.2
   Compiling chrono v0.4.44
   Compiling crossbeam-channel v0.5.15
   Compiling plotters-svg v0.3.7
   Compiling core-foundation v0.10.1
   Compiling minipaste v0.1.0
   Compiling ttf-parser v0.20.0
   Compiling ttf-parser v0.25.1
   Compiling tracing-subscriber v0.3.23
   Compiling synstructure v0.13.2
   Compiling plotters-bitmap v0.3.7
   Compiling zerofrom-derive v0.1.6
   Compiling yoke-derive v0.8.1
   Compiling zerovec-derive v0.11.2
   Compiling displaydoc v0.2.5
   Compiling foreign-types-macros v0.2.3
   Compiling foreign-types v0.5.0
   Compiling serde_derive v1.0.228
   Compiling tracing-attributes v0.1.31
   Compiling zerofrom v0.1.6
   Compiling core-graphics v0.23.2
   Compiling fltk v1.5.22
   Compiling core-text v20.1.0
   Compiling yoke v0.8.1
   Compiling tracing v0.1.44
   Compiling zerovec v0.11.5
   Compiling zerotrie v0.2.3
   Compiling plotters v0.3.7
   Compiling serde v1.0.228
   Compiling tinystr v0.8.2
   Compiling potential_utf v0.1.4
   Compiling icu_collections v2.1.1
   Compiling icu_locale_core v2.1.1
   Compiling if97_core v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/core)
   Compiling if97_app_api v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/if97_app_api)
   Compiling icu_provider v2.1.1
   Compiling icu_properties v2.1.2
   Compiling icu_normalizer v2.1.1
   Compiling idna_adapter v1.2.1
   Compiling idna v1.1.0
   Compiling url v2.5.8
   Compiling webbrowser v1.2.0
   Compiling if97_calculator_fltk v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/fltk)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.16s
     Running unittests src/lib.rs (target/debug/deps/if97_calculator_fltk-f160a1cc8dcc6044)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/if97_calculator_fltk-4428b0b00f5aac2f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests if97_calculator_fltk

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Оценка результата:
- `OK`: пакет собирается (unit-тестов нет).
- `NOTE`: в offline/sandbox средах возможен ожидаемый fail из-за `fltk-bundled` (скачивание bundled libs).

### if97_calculator_native (Tauri backend)

Команда:
```bash
cargo test -p if97_calculator_native --frozen --color never -- --nocapture
```

Вывод:
```text
   Compiling proc-macro2 v1.0.106
   Compiling serde_core v1.0.228
   Compiling zerocopy v0.8.47
   Compiling serde v1.0.228
   Compiling getrandom v0.2.17
   Compiling stable_deref_trait v1.2.1
   Compiling siphasher v1.0.2
   Compiling siphasher v0.3.11
   Compiling rand_core v0.6.4
   Compiling phf_shared v0.11.3
   Compiling getrandom v0.1.16
   Compiling typeid v1.0.3
   Compiling serde_json v1.0.149
   Compiling winnow v1.0.0
   Compiling syn v1.0.109
   Compiling strsim v0.11.1
   Compiling ident_case v1.0.1
   Compiling toml_parser v1.1.0+spec-1.1.0
   Compiling quote v1.0.45
   Compiling syn v2.0.117
   Compiling rand_core v0.5.1
   Compiling winnow v0.7.15
   Compiling erased-serde v0.4.10
   Compiling toml_writer v1.1.0+spec-1.1.0
   Compiling thiserror v2.0.18
   Compiling rand_pcg v0.2.1
   Compiling ppv-lite86 v0.2.21
   Compiling phf_shared v0.8.0
   Compiling aho-corasick v1.1.4
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.2.2
   Compiling rand v0.8.5
   Compiling parking_lot_core v0.9.12
   Compiling regex-syntax v0.8.10
   Compiling unic-char-range v0.9.0
   Compiling thiserror v1.0.69
   Compiling proc-macro-hack v0.5.20+deprecated
   Compiling unic-common v0.9.0
   Compiling unic-ucd-version v0.9.0
   Compiling unic-char-property v0.9.0
   Compiling phf_generator v0.11.3
   Compiling rand v0.7.3
   Compiling phf_shared v0.10.0
   Compiling new_debug_unreachable v1.0.6
   Compiling alloc-no-stdlib v2.0.4
   Compiling scopeguard v1.2.0
   Compiling synstructure v0.13.2
   Compiling darling_core v0.23.0
   Compiling phf_generator v0.8.0
   Compiling regex-automata v0.4.14
   Compiling lock_api v0.4.14
   Compiling alloc-stdlib v0.2.2
   Compiling phf_generator v0.10.0
   Compiling phf_codegen v0.11.3
   Compiling string_cache_codegen v0.5.4
   Compiling unic-ucd-ident v0.9.0
   Compiling mac v0.1.1
   Compiling precomputed-hash v0.1.1
   Compiling anyhow v1.0.102
   Compiling futf v0.1.5
   Compiling parking_lot v0.12.5
   Compiling markup5ever v0.14.1
   Compiling brotli-decompressor v5.0.0
   Compiling getrandom v0.4.2
   Compiling phf_codegen v0.8.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.0
   Compiling semver v1.0.27
   Compiling num-conv v0.2.1
   Compiling zerofrom-derive v0.1.6
   Compiling serde_derive v1.0.228
   Compiling yoke-derive v0.8.1
   Compiling zerovec-derive v0.11.2
   Compiling displaydoc v0.2.5
   Compiling phf_macros v0.11.3
   Compiling darling_macro v0.23.0
   Compiling zerofrom v0.1.6
   Compiling thiserror-impl v2.0.18
   Compiling thiserror-impl v1.0.69
   Compiling darling v0.23.0
   Compiling regex v1.12.3
   Compiling yoke v0.8.1
   Compiling phf_macros v0.10.0
   Compiling cssparser v0.29.6
   Compiling serde_with_macros v3.18.0
   Compiling zerovec v0.11.5
   Compiling zerotrie v0.2.3
   Compiling dunce v1.0.5
   Compiling powerfmt v0.2.0
   Compiling percent-encoding v2.3.2
   Compiling utf-8 v0.7.6
   Compiling dtoa v1.0.11
   Compiling tinystr v0.8.2
   Compiling potential_utf v0.1.4
   Compiling time-core v0.1.8
   Compiling indexmap v1.9.3
   Compiling icu_locale_core v2.1.1
   Compiling icu_collections v2.1.1
   Compiling dtoa-short v0.3.5
   Compiling tendril v0.4.3
   Compiling form_urlencoded v1.2.2
   Compiling deranged v0.5.8
   Compiling phf v0.10.1
   Compiling phf v0.11.3
   Compiling brotli v8.0.2
   Compiling cssparser-macros v0.6.1
   Compiling icu_provider v2.1.1
   Compiling string_cache v0.8.9
   Compiling ctor v0.2.9
   Compiling icu_normalizer v2.1.1
   Compiling icu_properties v2.1.2
   Compiling toml v0.9.12+spec-1.1.0
   Compiling selectors v0.24.0
   Compiling uuid v1.23.0
   Compiling camino v1.2.2
   Compiling glob v0.3.3
   Compiling convert_case v0.4.0
   Compiling matches v0.1.10
   Compiling nodrop v0.1.14
   Compiling servo_arc v0.2.0
   Compiling derive_more v0.99.20
   Compiling idna_adapter v1.2.1
   Compiling idna v1.1.0
   Compiling swift-rs v1.0.7
   Compiling url v2.5.8
   Compiling match_token v0.1.0
   Compiling serde_derive_internals v0.29.1
   Compiling phf v0.8.0
   Compiling fxhash v0.2.1
   Compiling objc2-exception-helper v0.1.1
   Compiling schemars v0.8.22
   Compiling hashbrown v0.12.3
   Compiling html5ever v0.29.1
   Compiling schemars_derive v0.8.22
   Compiling cfb v0.7.3
   Compiling jsonptr v0.6.3
   Compiling cargo-platform v0.1.9
   Compiling dyn-clone v1.0.20
   Compiling base64 v0.21.7
   Compiling cargo_metadata v0.19.2
   Compiling infer v0.19.0
   Compiling json-patch v3.0.1
   Compiling serde-untagged v0.1.9
   Compiling urlpattern v0.3.0
   Compiling serde_with v3.18.0
   Compiling bitflags v2.11.0
   Compiling version_check v0.9.5
   Compiling objc2 v0.6.4
   Compiling objc2-encode v4.1.0
   Compiling kuchikiki v0.8.8-speedreader
   Compiling quick-xml v0.38.4
   Compiling time v0.3.47
   Compiling rustc_version v0.4.1
   Compiling embed-resource v3.0.8
   Compiling typenum v1.19.0
   Compiling heck v0.5.0
   Compiling tauri-winres v0.3.5
   Compiling generic-array v0.14.7
   Compiling objc2-core-foundation v0.3.2
   Compiling block2 v0.6.2
   Compiling cargo_toml v0.22.3
   Compiling tauri-utils v2.8.3
   Compiling plist v1.8.0
   Compiling objc2-foundation v0.3.2
   Compiling time-macros v0.2.27
   Compiling raw-window-handle v0.6.2
   Compiling cookie v0.18.1
   Compiling dpi v0.1.2
   Compiling core-foundation-sys v0.8.7
   Compiling core-foundation v0.10.1
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling foreign-types-macros v0.2.3
   Compiling digest v0.10.7
   Compiling foreign-types v0.5.0
   Compiling core-graphics-types v0.2.0
   Compiling dispatch2 v0.3.1
   Compiling cpufeatures v0.2.17
   Compiling wry v0.54.4
   Compiling tauri-runtime v2.10.1
   Compiling sha2 v0.10.9
   Compiling core-graphics v0.25.0
   Compiling ico v0.5.0
   Compiling tauri-runtime-wry v2.10.1
   Compiling unicode-segmentation v1.13.2
   Compiling getrandom v0.3.4
   Compiling keyboard-types v0.7.0
   Compiling serialize-to-javascript-impl v0.1.2
   Compiling signal-hook v0.3.18
   Compiling tokio v1.50.0
   Compiling objc2-app-kit v0.3.2
   Compiling serialize-to-javascript v0.1.2
   Compiling serde_repr v0.1.20
   Compiling embed_plist v1.2.2
   Compiling os_pipe v1.2.3
   Compiling rfd v0.16.0
   Compiling sigchld v0.2.4
   Compiling tracing-attributes v0.1.31
   Compiling pathdiff v0.2.3
   Compiling open v5.3.3
   Compiling shared_child v1.1.1
   Compiling encoding_rs v0.8.35
   Compiling if97_app_api v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/if97_app_api)
   Compiling tracing v0.1.44
   Compiling tauri-build v2.5.6
   Compiling tauri-plugin v2.5.4
   Compiling tauri-codegen v2.5.5
   Compiling if97_core v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/core)
   Compiling objc2-web-kit v0.3.2
   Compiling tao v0.34.8
   Compiling window-vibrancy v0.6.0
   Compiling muda v0.17.1
   Compiling tauri v2.10.3
   Compiling tauri-plugin-fs v2.4.5
   Compiling tauri-macros v2.5.5
   Compiling tauri-plugin-shell v2.3.5
   Compiling tauri-plugin-dialog v2.6.0
   Compiling if97_calculator_native v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/wasm/src-tauri)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 54.68s
     Running unittests src/lib.rs (target/debug/deps/app_lib-dc9121f493ab9578)

running 4 tests
test dome_perf_tests::dome_perf_all_diagrams ... ignored
test smoke_tests::calculate_state_smoke_pt_ok ... ok
test smoke_tests::calculate_table_rows_smoke_parses_mixed_separators ... ok
test smoke_tests::calculate_dome_points_smoke_non_empty ... ok

test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src/main.rs (target/debug/deps/if97_calculator_native-c345e8887364f86b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests app_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Оценка результата:
- `OK`: smoke-тесты backend-команд проходят; perf-тест купола насыщения остаётся ignored.

### if97_calculator_web (WASM/Yew) compile-check

Команда:
```bash
cargo check -p if97_calculator_web --target wasm32-unknown-unknown --frozen --color never
```

Вывод:
```text
   Compiling unicode-ident v1.0.24
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling wasm-bindgen-shared v0.2.115
   Compiling rustversion v1.0.22
    Checking once_cell v1.21.4
    Checking pin-project-lite v0.2.17
    Checking memchr v2.8.0
    Checking futures-core v0.3.32
    Checking cfg-if v1.0.4
    Checking futures-sink v0.3.32
   Compiling bumpalo v3.20.2
    Checking futures-channel v0.3.32
    Checking slab v0.4.12
    Checking futures-task v0.3.32
    Checking futures-io v0.3.32
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling zmij v1.0.21
    Checking itoa v1.0.18
   Compiling serde_json v1.0.149
   Compiling thiserror v1.0.69
    Checking percent-encoding v2.3.2
   Compiling version_check v0.9.5
    Checking form_urlencoded v1.2.2
    Checking ryu v1.0.23
    Checking fnv v1.0.7
   Compiling syn v2.0.117
    Checking bytes v1.11.1
   Compiling wasm-bindgen v0.2.115
   Compiling toml_datetime v0.6.3
    Checking http v0.2.12
   Compiling winnow v0.5.40
   Compiling proc-macro-error-attr v1.0.4
   Compiling syn v1.0.109
   Compiling autocfg v1.5.0
   Compiling num-traits v0.2.19
   Compiling proc-macro-error v1.0.4
    Checking tracing-core v0.1.36
    Checking anymap2 v0.13.0
    Checking plotters-backend v0.3.7
   Compiling prettyplease v0.2.37
   Compiling toml_edit v0.19.15
   Compiling wasm-bindgen-macro-support v0.2.115
    Checking weezl v0.1.12
    Checking hashbrown v0.16.1
    Checking color_quant v1.1.0
    Checking equivalent v1.0.2
    Checking gif v0.12.0
    Checking lazy_static v1.5.0
   Compiling proc-macro-crate v1.3.1
    Checking indexmap v2.13.0
    Checking log v0.4.29
   Compiling boolinator v2.4.0
    Checking tracing-log v0.2.0
    Checking sharded-slab v0.1.7
    Checking plotters-bitmap v0.3.7
    Checking plotters-svg v0.3.7
    Checking thread_local v1.1.9
    Checking smallvec v1.15.1
    Checking nu-ansi-term v0.50.3
    Checking tracing-subscriber v0.3.23
   Compiling futures-macro v0.3.32
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v1.0.69
   Compiling pin-project-internal v1.1.11
   Compiling gloo-worker-macros v0.1.0
   Compiling implicit-clone-derive v0.1.2
   Compiling tracing-attributes v0.1.31
    Checking implicit-clone v0.4.9
    Checking futures-util v0.3.32
   Compiling yew-macro v0.21.0
   Compiling wasm-bindgen-macro v0.2.115
    Checking pin-project v1.1.11
    Checking tracing v0.1.44
    Checking console_error_panic_hook v0.1.7
    Checking js-sys v0.3.92
    Checking futures v0.3.32
    Checking pinned v0.1.0
    Checking serde_urlencoded v0.7.1
    Checking bincode v1.3.3
    Checking if97_app_api v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/if97_app_api)
    Checking web-sys v0.3.92
    Checking wasm-bindgen-futures v0.4.65
    Checking serde-wasm-bindgen v0.5.0
    Checking getrandom v0.2.17
    Checking serde-wasm-bindgen v0.6.5
    Checking gloo-timers v0.2.6
    Checking gloo-timers v0.3.0
    Checking chrono v0.4.44
    Checking gloo-utils v0.1.7
    Checking gloo-utils v0.2.0
    Checking gloo-events v0.1.2
    Checking gloo-events v0.2.0
    Checking gloo-dialogs v0.1.1
    Checking gloo-render v0.1.1
    Checking gloo-dialogs v0.2.0
    Checking gloo-render v0.2.0
    Checking plotters v0.3.7
    Checking plotters-canvas v0.3.1
    Checking gloo-file v0.2.3
    Checking gloo-console v0.3.0
    Checking gloo-file v0.3.0
    Checking gloo-history v0.2.2
    Checking gloo-net v0.4.0
    Checking gloo-console v0.2.3
    Checking gloo-storage v0.2.2
    Checking gloo-history v0.1.5
    Checking gloo-net v0.3.1
    Checking gloo-storage v0.3.0
    Checking gloo-worker v0.4.0
    Checking gloo-worker v0.2.1
    Checking gloo v0.10.0
    Checking gloo v0.8.1
    Checking prokio v0.1.0
    Checking yew v0.21.0
    Checking if97_calculator_web v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/wasm)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 09s
```

Оценка результата:
- `OK`: wasm32 сборка проходит.

### Web/WASM: trunk build --release

Команда:
```bash
cd wasm && NO_COLOR=true trunk build --release
```

Вывод:
```text
2026-04-09T10:48:49.400135Z  INFO 🚀 Starting trunk 0.21.14
2026-04-09T10:48:50.432563Z  INFO 📦 starting build
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling wasm-bindgen-shared v0.2.115
   Compiling rustversion v1.0.22
   Compiling once_cell v1.21.4
   Compiling bumpalo v3.20.2
   Compiling futures-sink v0.3.32
   Compiling futures-core v0.3.32
   Compiling cfg-if v1.0.4
   Compiling memchr v2.8.0
   Compiling pin-project-lite v0.2.17
   Compiling futures-channel v0.3.32
   Compiling futures-task v0.3.32
   Compiling futures-io v0.3.32
   Compiling slab v0.4.12
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling itoa v1.0.18
   Compiling zmij v1.0.21
   Compiling serde_json v1.0.149
   Compiling thiserror v1.0.69
   Compiling syn v2.0.117
   Compiling percent-encoding v2.3.2
   Compiling version_check v0.9.5
   Compiling wasm-bindgen v0.2.115
   Compiling form_urlencoded v1.2.2
   Compiling hashbrown v0.16.1
   Compiling equivalent v1.0.2
   Compiling fnv v1.0.7
   Compiling ryu v1.0.23
   Compiling bytes v1.11.1
   Compiling winnow v0.5.40
   Compiling toml_datetime v0.6.3
   Compiling proc-macro-error-attr v1.0.4
   Compiling syn v1.0.109
   Compiling autocfg v1.5.0
   Compiling indexmap v2.13.0
   Compiling http v0.2.12
   Compiling num-traits v0.2.19
   Compiling proc-macro-error v1.0.4
   Compiling tracing-core v0.1.36
   Compiling plotters-backend v0.3.7
   Compiling prettyplease v0.2.37
   Compiling anymap2 v0.13.0
   Compiling color_quant v1.1.0
   Compiling weezl v0.1.12
   Compiling toml_edit v0.19.15
   Compiling gif v0.12.0
   Compiling boolinator v2.4.0
   Compiling lazy_static v1.5.0
   Compiling log v0.4.29
   Compiling tracing-log v0.2.0
   Compiling sharded-slab v0.1.7
   Compiling plotters-bitmap v0.3.7
   Compiling plotters-svg v0.3.7
   Compiling thread_local v1.1.9
   Compiling smallvec v1.15.1
   Compiling nu-ansi-term v0.50.3
   Compiling tracing-subscriber v0.3.23
   Compiling wasm-bindgen-macro-support v0.2.115
   Compiling proc-macro-crate v1.3.1
   Compiling futures-macro v0.3.32
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v1.0.69
   Compiling pin-project-internal v1.1.11
   Compiling tracing-attributes v0.1.31
   Compiling futures-util v0.3.32
   Compiling gloo-worker-macros v0.1.0
   Compiling pin-project v1.1.11
   Compiling implicit-clone-derive v0.1.2
   Compiling yew-macro v0.21.0
   Compiling tracing v0.1.44
   Compiling implicit-clone v0.4.9
   Compiling wasm-bindgen-macro v0.2.115
   Compiling bincode v1.3.3
   Compiling serde_urlencoded v0.7.1
   Compiling if97_app_api v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/if97_app_api)
   Compiling futures v0.3.32
   Compiling pinned v0.1.0
   Compiling js-sys v0.3.92
   Compiling console_error_panic_hook v0.1.7
   Compiling web-sys v0.3.92
   Compiling wasm-bindgen-futures v0.4.65
   Compiling serde-wasm-bindgen v0.5.0
   Compiling gloo-timers v0.2.6
   Compiling serde-wasm-bindgen v0.6.5
   Compiling getrandom v0.2.17
   Compiling gloo-timers v0.3.0
   Compiling chrono v0.4.44
   Compiling gloo-utils v0.1.7
   Compiling gloo-utils v0.2.0
   Compiling gloo-events v0.1.2
   Compiling gloo-events v0.2.0
   Compiling gloo-dialogs v0.1.1
   Compiling gloo-render v0.1.1
   Compiling gloo-render v0.2.0
   Compiling gloo-dialogs v0.2.0
   Compiling plotters v0.3.7
   Compiling plotters-canvas v0.3.1
   Compiling gloo-console v0.2.3
   Compiling gloo-net v0.3.1
   Compiling gloo-history v0.1.5
   Compiling gloo-storage v0.2.2
   Compiling gloo-file v0.2.3
   Compiling gloo-storage v0.3.0
   Compiling gloo-worker v0.2.1
   Compiling gloo-net v0.4.0
   Compiling gloo-history v0.2.2
   Compiling gloo-console v0.3.0
   Compiling gloo-worker v0.4.0
   Compiling gloo v0.8.1
   Compiling gloo-file v0.3.0
   Compiling prokio v0.1.0
   Compiling gloo v0.10.0
   Compiling yew v0.21.0
   Compiling if97_calculator_web v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/wasm)
    Finished `release` profile [optimized] target(s) in 1m 07s
2026-04-09T10:49:59.199292Z  INFO applying new distribution
2026-04-09T10:49:59.201148Z  INFO ✅ success
```

Оценка результата:
- `OK/FAIL`: зависит от окружения; при `NO_COLOR=1` ожидаемо падает.

## Benchmarks

Ниже: протокол прогонов, входные наборы и статистика (mean, sample stdev, RSD, 95% CI по t-распределению).

### Bench 1: CPU нагрузка ядра из if97_core (ignored test)

Команда:
```bash
IF97_LOAD_ITERS=200 cargo test -p if97_core --test load_stress_test --frozen --color never -- --ignored --nocapture
```

Вывод:
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running tests/load_stress_test.rs (target/debug/deps/load_stress_test-3ea53ec8787a7cff)

running 1 test
load_stress_core_csv: rows=242 iters=200 total_rows=48400 elapsed=24.095s rows/s=2009 calls/s=8035 ok=190800 err=2800 checksum=1526026781224467136
test load_stress_core_csv ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.10s

```

Входные данные:
- Встроенный CSV `core/tests/if97_rust_test_data.csv`.

Оценка результата:
- `OK`: стресс-тест выполняется и печатает throughput/checksum.

### Bench 2 (build): if97_loadtest

Команда:
```bash
cargo build -p if97_calculator_service --bin if97_loadtest --features loadtest --frozen --color never
```

Вывод:
```text
   Compiling serde v1.0.228
   Compiling if97_calculator_service v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/service)
   Compiling csv-core v0.1.13
   Compiling csv v1.4.0
   Compiling axum v0.7.9
   Compiling if97_core v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/core)
   Compiling tonic v0.12.3
   Compiling tonic-health v0.12.3
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.32s
```


### Bench 2: CPU нагрузка ядра (multi-thread) через if97_loadtest

Команда (серия из 10 прогонов, строго последовательно):
```bash
./target/debug/if97_loadtest --iters 200 --threads 8
```

Raw результаты (n=10):
```text
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=9.274s rows/s=5219 calls/s=20875 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=9.902s rows/s=4888 calls/s=19552 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=10.236s rows/s=4728 calls/s=18913 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=9.557s rows/s=5064 calls/s=20257 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=9.384s rows/s=5158 calls/s=20630 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=9.153s rows/s=5288 calls/s=21152 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=8.303s rows/s=5829 calls/s=23316 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=8.368s rows/s=5784 calls/s=23136 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=7.744s rows/s=6250 calls/s=25000 ok=190800 err=2800 checksum=1526026781224467136
if97_loadtest: profile=debug rows=242 iters=200 threads=8 total_rows=48400 elapsed=7.673s rows/s=6308 calls/s=25230 ok=190800 err=2800 checksum=1526026781224467136
```

Сводка по throughput `rows/s` (t-based 95% CI, df=9):
- mean: `5451.6 rows/s`
- sample stdev: `556.01 rows/s`
- RSD: `10.20%`
- 95% CI: `[5053.88, 5849.32] rows/s`
- min/max: `[4728, 6308] rows/s`

### Bench 3: генерация купола насыщения (Tauri, ignored perf-test)

Команда:
```bash
cargo test -p if97_calculator_native --frozen --color never -- --ignored --nocapture
```

Вывод:
```text
   Compiling if97_calculator_native v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/wasm/src-tauri)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.00s
     Running unittests src/lib.rs (target/debug/deps/app_lib-dc9121f493ab9578)

running 1 test
dome_perf: chart=Pt swap_axes=false points=717 elapsed=0.024s
dome_perf: chart=Pt swap_axes=true points=717 elapsed=0.024s
dome_perf: chart=Pv swap_axes=false points=4249 elapsed=0.097s
dome_perf: chart=Pv swap_axes=true points=4249 elapsed=0.097s
dome_perf: chart=Tv swap_axes=false points=2396 elapsed=0.068s
dome_perf: chart=Tv swap_axes=true points=2396 elapsed=0.068s
dome_perf: chart=Ph swap_axes=false points=1739 elapsed=0.067s
dome_perf: chart=Ph swap_axes=true points=1739 elapsed=0.068s
dome_perf: chart=Ps swap_axes=false points=1586 elapsed=0.062s
dome_perf: chart=Ps swap_axes=true points=1586 elapsed=0.062s
dome_perf: chart=Ts swap_axes=false points=825 elapsed=0.032s
dome_perf: chart=Ts swap_axes=true points=825 elapsed=0.032s
dome_perf: chart=Th swap_axes=false points=861 elapsed=0.030s
dome_perf: chart=Th swap_axes=true points=861 elapsed=0.030s
dome_perf: chart=Hs swap_axes=false points=1148 elapsed=0.040s
dome_perf: chart=Hs swap_axes=true points=1148 elapsed=0.040s
test dome_perf_tests::dome_perf_all_diagrams ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.84s

     Running unittests src/main.rs (target/debug/deps/if97_calculator_native-c345e8887364f86b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests app_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```


### Bench 4: gRPC транспорт (service + grpc_loadtest)

Примечание: в sandbox-режиме запуск gRPC-сервиса может падать с `PermissionDenied` (локальный `listen` запрещён). В этом отчете gRPC-прогон выполнен вне песочницы.

### Bench 4 (build): service + grpc_loadtest + grpc_healthcheck

Команда:
```bash
cargo build -p if97_calculator_service --bin if97_calculator_service --bin grpc_loadtest --bin grpc_healthcheck --frozen --color never
```

Вывод:
```text
   Compiling if97_calculator_service v1.1.0 (/private/tmp/if97_calculator_tests.33J7CY/repo/service)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.76s
```


#### 4.1 Запуск сервиса (локально)

Команда:
```bash
IF97_GRPC_ADDR=127.0.0.1:50051 \
IF97_KERNEL=core \
IF97_IO_THREADS=2 \
IF97_CPU_THREADS=8 \
IF97_MAX_IN_FLIGHT_PER_CONN=8 \
IF97_MAX_IN_FLIGHT_GLOBAL=8 \
IF97_DRAIN_SECS=2 \
RUST_LOG=info \
./target/debug/if97_calculator_service
```

Вывод:
```text
2026-04-09T10:52:15.082867Z  INFO if97_calculator_service: if97_calculator_service listening on 127.0.0.1:50051
service: started (tcp ready)
```

Health-check (прямой, через gRPC Health API):
```text
grpc_healthcheck: addr=127.0.0.1:50051 service="if97.v1.If97Service" status=Serving
grpc_healthcheck: addr=127.0.0.1:50051 service="" status=Serving
```

#### 4.2 Серия замеров (streams=1, batches=500)

Команда:
```bash
./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 500 --batch-size 2048 --in-flight 8
```

Raw результаты (n=6):
```text
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=500 batch_size=2048 total_batches=500 elapsed=8.374s batches/s=60 points/s=122285
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=500 batch_size=2048 total_batches=500 elapsed=8.046s batches/s=62 points/s=127275
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=500 batch_size=2048 total_batches=500 elapsed=7.504s batches/s=67 points/s=136464
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=500 batch_size=2048 total_batches=500 elapsed=7.579s batches/s=66 points/s=135110
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=500 batch_size=2048 total_batches=500 elapsed=7.703s batches/s=65 points/s=132937
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=500 batch_size=2048 total_batches=500 elapsed=8.276s batches/s=60 points/s=123726
```

Сводка по throughput `points/s` (t-based 95% CI, df=5):
- mean: `129632.83 points/s`
- sample stdev: `6033.61 points/s`
- RSD: `4.65%`
- 95% CI: `[123299.91, 135965.75] points/s`
- min/max: `[122285, 136464] points/s`

#### 4.3 Длинный прогон (batches=2000, один запуск)

Команда:
```bash
./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 2000 --batch-size 2048 --in-flight 8
```

Вывод:
```text
grpc_loadtest: profile=debug addr=127.0.0.1:50051 streams=1 batches=2000 batch_size=2048 total_batches=2000 elapsed=33.337s batches/s=60 points/s=122865
```


## Пакет демонстрационных данных (CSV)

### Контрольные точки IF97 (численные reference points)

Файл: `core/tests/if97_rust_test_data.csv`
- Формат: CSV с заголовком.
- Колонки (факт заголовка):
  `P_MPa,T_K,Region,x_vapor_fraction,v_m3_kg,rho_kg_m3,h_kJ_kg,s_kJ_kgK,u_kJ_kg,cp_kJ_kgK,cv_kJ_kgK,w_m_s`
- Валидных строк данных: `242` (покрыты регионы `1..5`).
- В конце файла есть 1 битая строка (состоит из одних запятых, не оканчивается `\n`), она игнорируется тестами из-за `flexible(true)` и `Err(_) => continue`.
- Распределение по регионам (по колонке `Region`): `{1: 53, 2: 64, 3: 3, 4: 100, 5: 22}`.
- Диапазоны: `p` от `0.000611657...` до `100.0 MPa`, `T` от `273.16 K` до `2000.0 K`.

Где используется:
- `core/tests/integration_csv_test.rs` (интеграционный тест).
- `core/tests/load_stress_test.rs` (ignored стресс-тест).
- `service/src/bin/if97_loadtest.rs` (CPU бенчмаркер).

Пример строки Region 4 (с фазовой долей `x`):
```text
P_MPa=0.0006116570000106653, T_K=273.16, x_vapor_fraction=0.0
```

### Таблицы коэффициентов (модели регионов и обратные функции)

Расположение: `core/data/*.csv` (коэффициенты уравнений, не контрольные точки).

Линейные размеры (wc -l, может недосчитать последнюю строку без `\n`):
```text
      20 core/data/backward1_t_ph.csv
      20 core/data/backward1_t_ps.csv
      34 core/data/backward2a_t_ph.csv
      46 core/data/backward2a_t_ps.csv
      38 core/data/backward2b_t_ph.csv
      44 core/data/backward2b_t_ps.csv
      23 core/data/backward2c_t_ph.csv
      30 core/data/backward2c_t_ps.csv
      34 core/data/region1.csv
      43 core/data/region2.csv
       9 core/data/region2_cp0.csv
      13 core/data/region2_meta.csv
      39 core/data/region3.csv
       6 core/data/region5.csv
       6 core/data/region5_cp0.csv
```

### Мини-набор граничных случаев (для ручной проверки UI)

Источник expected-значений: `core/src/tests.rs` (официальные verification table + границы насыщения).

Рекомендуемые входы для UI (p-T):
```text
p_MPa;T_K
3.0;300.0
80.0;300.0
3.0;500.0
0.0035;300.0
0.0035;700.0
30.0;700.0
0.5;1500.0
30.0;2000.0
```

Рекомендуемые входы для UI (saturation check, p->Tsat из `test_region4_saturation_lines`):
```text
p_MPa;T_sat_expected_K
0.1;372.755919
1.0;453.035632
10.0;584.149488
```

## Матрица трассировки (введение -> модуль -> тест/артефакт)

| Задача из README ("Что умеет проект") | Реализованный модуль | Тест/артефакт | Примечание/доказательство |
|---|---|---|---|
| Расчеты по входным парам `p-T`, `p-h`, `p-s`, `p-x`, `rho-T` | `core/` (`If97::{pt,ph,ps,px,rhot}`) | `core/src/tests.rs`, `core/tests/integration_csv_test.rs`, `core/tests/if97_rust_test_data.csv` | Интеграционный тест печатает счетчики: `pT=142`, `rhoT=142`, `px=100`, `ph=236`, `ps=236` |
| Табличный расчет с автопересчетом, импортом и экспортом | `fltk/src/ui/batch.rs`, `wasm/src/ui/table_calc.rs`, Tauri команды `load_file_dialog/save_file_dialog` | Demo-сценарии (см. ниже) | Автотаймаут пересчета в web, Table UI+экспорт CSV в FLTK |
| Сохранение точек и таблиц для последующей отрисовки | `fltk/src/state.rs` (`SavedData`), `wasm/src/types.rs` (`SavedItem`) | Demo-сценарии (см. ниже) | Сохраненные наборы участвуют в графиках |
| Диаграммы `pT`, `pV`, `pS`, `pH`, `TV`, `TS`, `TH`, `HS` | `fltk/src/ui/plot_tab.rs`, `wasm/src/ui/plots.rs`, `wasm/src-tauri/src/lib.rs` (dome) | `cargo test -p if97_calculator_native -- --ignored --nocapture` | Perf-тест `dome_perf_all_diagrams` прогоняет все диаграммы (Pt/Pv/Tv/Ph/Ps/Ts/Th/Hs) |
| Системный журнал с экспортом в `.log` | `fltk/src/ui/about.rs` (`SaveLogFile`), `wasm/src/ui/about_logs.rs` (`Экспорт .log`) | Demo-сценарии (см. ниже) | Экспорт реализован через диалоги сохранения файла |
| Серверный режим для горизонтального масштабирования вычислений | `service/` (tonic + rayon) | `service/Dockerfile`, `service/k8s/*.yaml`, `grpc_loadtest`, `tonic_health` | В k8s Deployment есть grpc probes и drain |

## Demo: сценарии и "скриншот-лист"

Примечание:
- Автоматически сгенерировать скриншоты в рамках этого прогона невозможно (нет управляемого GUI/захвата экрана как артефактов при запрете создавать файлы кроме `TESTS.md`).
- Ниже приведены воспроизводимые сценарии + список скриншотов, которые стоит снять вручную (macOS: `Cmd+Shift+4` или `Cmd+Shift+5`).

### FLTK desktop (`if97_calculator_fltk`)

Сборка/запуск:
```bash
cargo run -p if97_calculator_fltk --release
```

Сценарий 1: одиночный расчет
- Вкладка "Одиночный расчет".
- Режим `p-T`, ввод: `p=3.0`, `T=300.0`.
- Ожидаемо: регион `Region1` и значения, близкие к unit-тестам (энтальпия около `115.331 kJ/kg`).

Сценарий 2: табличный расчет + экспорт CSV
- Вкладка "Табличный расчет".
- Вставить 5-10 строк пар значений (например из мини-набора выше, раздел CSV).
- Проверить подсветку ошибок/валидных строк, выбрать точность.
- Нажать "Экспорт CSV" и проверить сохраненный файл.

Сценарий 3: графики + купол + экспорт PNG
- Сохранить 1-2 точки и 1 таблицу.
- Вкладка "Графики": выбрать тип диаграммы, включить/выключить купол, swap axes, автомасштаб/ручной масштаб.
- Нажать "Экспорт в PNG" и проверить PNG.

Сценарий 4: логи + экспорт .log
- Вкладка "О программе": "Сохранить логи работы (.log)".

Список скриншотов (рекомендация):
- FLTK: одиночный расчет с заполненными полями и результатом.
- FLTK: табличный расчет с 10-20 строками и видимой таблицей результатов.
- FLTK: график `p-T` с включенным куполом и выбранными datasets.
- FLTK: диалог/результат экспорта CSV.
- FLTK: диалог/результат экспорта PNG.
- FLTK: экспорт логов (.log) или окно "О программе" с кнопкой.

### Yew/WASM + Tauri (`if97_calculator_web` + `if97_calculator_native`)

Web (browser) запуск:
```bash
cd wasm
NO_COLOR=true trunk serve
```

Tauri (desktop) запуск:
```bash
cd wasm
NO_COLOR=true cargo tauri dev
```

Сценарий 1: Single calc
- Вкладка одиночного расчета.
- Переключить режимы `p-T`, `rho-T`, `p-h`, `p-s`, `p-x`.
- Проверить форматирование нечисел (`∞`, `NaN`) при соответствующих входах/граничных точках (DTO умеет сериализовать).

Сценарий 2: Table calc + генератор + экспорт
- Вкладка табличного расчета: вставка вручную (`value1;value2`), проверка автопересчета.
- Использовать генератор (диапазоны/шаги), проверить лимит `MAX_GENERATED_ROWS`.
- Экспорт результатов (через диалог сохранения).

Сценарий 3: Plots (canvas) + купол + zoom/pan + экспорт PNG
- Сохранить несколько точек и таблицу.
- Вкладка графиков: выбрать диаграмму, включить купол, swap axes, проверить управление диапазонами.
- Экспорт в PNG (Tauri команда `save_plot_dialog`).

Сценарий 4: About/Logs + экспорт .log
- Вкладка журнала: убедиться, что логи периодически обновляются (interval 1s).
- Экспорт `.log`.

Список скриншотов (рекомендация):
- Web: single calc (p-T) с результатом + сохранение точки.
- Web: table calc с 100+ строками (показ ограниченного количества строк) и выделением строки/колонки.
- Web: plots `p-T` с куполом и несколькими сериями.
- Web/Tauri: диалог сохранения CSV/PNG/log и факт создания файла.
- Tauri: вкладка лога с экспортом.

## Runbook демонстрации gRPC сервиса (local / Docker / Kubernetes)

### Локально

Запуск:
```bash
IF97_GRPC_ADDR=0.0.0.0:50051 cargo run -p if97_calculator_service --bin if97_calculator_service
```

Проверка "живости" (functional):
```bash
./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 1 --batch-size 16 --in-flight 1
```

Проверка health API (grpc.health.v1.Health):
```bash
cargo run -p if97_calculator_service --bin grpc_healthcheck -- --addr 127.0.0.1:50051
```

Drain:
- Послать SIGTERM/SIGINT процессу.
- Ожидаемо: health переводится в `NotServing`, после чего выдерживается `IF97_DRAIN_SECS` секунд (по коду `service/src/lib.rs`).

### Docker

Сборка:
```bash
docker build -t if97-calculator-service:1.1.0 -f service/Dockerfile .
```

Запуск:
```bash
docker run --rm -p 50051:50051 if97-calculator-service:1.1.0
```

Нагрузочный тест внутри контейнера (как в `service/README.md`):
```bash
docker run --rm --entrypoint /usr/local/bin/grpc_loadtest if97-calculator-service:1.1.0 -- --addr 127.0.0.1:50051 --streams 1 --batches 2000 --batch-size 2048 --in-flight 8
```

### Kubernetes

Применение манифестов:
```bash
kubectl apply -f service/k8s/
```

Проверки:
- `service/k8s/deployment.yaml` содержит `readinessProbe`/`livenessProbe` типа `grpc` для сервиса `if97.v1.If97Service`.
- Drain: `terminationGracePeriodSeconds: 30` + `preStop: sleep 3`, плюс `IF97_DRAIN_SECS` (env) внутри приложения.
- Loadtest Job: `service/k8s/loadtest-job.yaml` запускает `/usr/local/bin/grpc_loadtest` в кластере.
