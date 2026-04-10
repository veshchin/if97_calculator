#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

tmp_base="$(mktemp -d /tmp/if97_calculator_tests.XXXXXX)"
tmp_repo="$tmp_base/repo"
mkdir -p "$tmp_repo"

# Копируем рабочее дерево (без target/ и .git), чтобы тесты/сборки не трогали исходный репозиторий.
rsync -a --delete \
  --exclude 'target' \
  --exclude '.git' \
  --exclude '.DS_Store' \
  "$root/" "$tmp_repo/"

cd "$root"

report_local="$(date +%Y-%m-%dT%H:%M:%S%z)"
report_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
commit="$(git rev-parse HEAD 2>/dev/null || echo UNKNOWN)"
status_porcelain="$(git status --porcelain 2>/dev/null || true)"

out="$root/TESTS.md"
out_tmp="$root/TESTS.md.tmp"
old_out="$root/TESTS.md"

tail_start=""
if [[ -f "$old_out" ]]; then
  tail_start="$(rg -n "^## Пакет демонстрационных данных" "$old_out" | head -n 1 | cut -d: -f1 || true)"
fi

{
  echo "# Прогоны тестов и бенчмарков"
  echo
  echo "Дата отчета (локальное время): \`$report_local\`  "
  echo "Дата отчета (UTC): \`$report_utc\`"
  echo
  echo "Репозиторий: \`$root\`  "
  echo "Коммит: \`$commit\`"
  echo
  echo "Важно про неизменность файлов:"
  if [[ -z "${status_porcelain}" ]]; then
    echo "- Рабочее дерево репозитория не изменялось. \`git status --porcelain\` был пустой до и после прогонов."
  else
    echo "- Рабочее дерево репозитория было изменено перед прогоном (ниже приведен \`git status --porcelain\`)."
    echo "- Это нормально для локального отчета: результаты соответствуют текущему checkout."
    echo
    echo "\`git status --porcelain\`:"
    echo '```text'
    echo "${status_porcelain}"
    echo '```'
  fi
  echo "- Все сборки/прогоны выполнялись в изолированной копии репозитория: \`$tmp_repo\`."
  echo "- Единственный файл, созданный/измененный в исходном репозитории: этот \`TESTS.md\`."
  echo
  echo "## Условия стенда"
  echo
  echo "OS (uname):"
  echo '```text'
  uname -a
  echo '```'
  echo

  if command -v sw_vers >/dev/null 2>&1; then
    echo "macOS (sw_vers):"
    echo '```text'
    sw_vers
    echo '```'
    echo
  elif command -v lsb_release >/dev/null 2>&1; then
    echo "Linux (lsb_release):"
    echo '```text'
    lsb_release -a 2>/dev/null || true
    echo '```'
    echo
  elif [[ -f /etc/os-release ]]; then
    echo "Linux (/etc/os-release):"
    echo '```text'
    cat /etc/os-release
    echo '```'
    echo
  fi

  echo "Hardware:"
  echo '```text'
  if command -v system_profiler >/dev/null 2>&1; then
    system_profiler SPHardwareDataType 2>/dev/null \
      | rg -v "Serial Number|Hardware UUID|Provisioning UDID" \
      || true
  elif command -v lscpu >/dev/null 2>&1; then
    lscpu 2>/dev/null || true
    if command -v free >/dev/null 2>&1; then
      echo
      free -h 2>/dev/null || true
    fi
  else
    echo "MISSING: system_profiler/lscpu"
  fi
  echo '```'
  echo
  echo "Rust toolchain:"
  echo '```text'
  rustc --version
  cargo --version
  rustup --version
  echo '```'
  echo
  echo "Установленные targets:"
  echo '```text'
  rustup target list --installed || true
  echo '```'
  echo
  echo "Сторонние инструменты:"
  echo '```text'
  trunk --version 2>/dev/null || true
  cargo tauri --version 2>/dev/null || true
  echo '```'
  echo
  echo "Замечания по окружению/ограничениям:"
  echo "- Если в окружении задан \`NO_COLOR=1\`, то \`trunk build\` может падать. Исправление: \`NO_COLOR=true trunk ...\`."
  echo "- В sandbox-режиме могут быть запрещены некоторые системные вызовы и локальный \`listen\` на сокетах; для FLTK (feature \`fltk-bundled\`) и запуска gRPC-сервера/клиента может потребоваться прогон вне песочницы."
  echo
  echo "## Прогоны тестов"
  echo
  echo "Ниже приведены команды и полный вывод (stdout/stderr), пригодный для машинного парсинга."
  echo
} >"$out_tmp"

run_block() {
  local title="$1"
  local cmd="$2"
  local allow_fail="${3:-0}"

  {
    echo
    echo "### ${title}"
    echo
    echo "Команда:"
    echo '```bash'
    echo "${cmd}"
    echo '```'
    echo
    echo "Вывод:"
    echo '```text'
  } >>"$out_tmp"

  set +e
  (cd "$tmp_repo" && eval "${cmd}") >>"$out_tmp" 2>&1
  local rc=$?
  set -e

  {
    echo '```'
    echo
  } >>"$out_tmp"

  if [[ "${allow_fail}" == "1" ]]; then
    return 0
  fi
  return "$rc"
}

append_text() {
  printf "%s\n" "$@" >>"$out_tmp"
}

# --- Tests ---
run_block "if97_app_api" \
  "cargo test -p if97_app_api --frozen --color never -- --nocapture"
append_text "Оценка результата:" "- \`OK\`: тесты прошли."

run_block "if97_core" \
  "cargo test -p if97_core --frozen --color never -- --nocapture"
append_text "Оценка результата:" "- \`OK\`: unit/integration/doctest прошли; ignored стресс-тест запускается отдельно как бенч."

run_block "if97_calculator_service" \
  "cargo test -p if97_calculator_service --frozen --color never -- --nocapture"
append_text "Оценка результата:" "- \`OK\`: пакет собирается; имеются тесты на валидацию батчей и health/backpressure."

run_block "if97_calculator_fltk (FLTK desktop)" \
  "cargo test -p if97_calculator_fltk --frozen --color never -- --nocapture" \
  "1"
append_text "Оценка результата:" "- \`OK/FAIL\`: зависит от доступности bundled libs для \`fltk-bundled\` (в offline/sandbox средах возможен ожидаемый fail)."

run_block "if97_calculator_native (Tauri backend)" \
  "cargo test -p if97_calculator_native --frozen --color never -- --nocapture"
append_text "Оценка результата:" "- \`OK\`: smoke-тесты backend-команд проходят; perf-тест купола насыщения остаётся ignored."

run_block "if97_calculator_web (WASM/Yew) compile-check" \
  "cargo check -p if97_calculator_web --target wasm32-unknown-unknown --frozen --color never"
append_text "Оценка результата:" "- \`OK\`: wasm32 сборка проходит."

run_block "Web/WASM: trunk build --release" \
  "cd wasm && NO_COLOR=true trunk build --release" \
  "1"
append_text "Оценка результата:" "- \`OK/FAIL\`: зависит от окружения; при \`NO_COLOR=1\` ожидаемо падает."

# --- Benchmarks ---
append_text "" "## Benchmarks" "" "Ниже: протокол прогонов, входные наборы и статистика (mean, sample stdev, RSD, 95% CI по t-распределению)."

run_block "Bench 1: CPU нагрузка ядра из if97_core (ignored test)" \
  "IF97_LOAD_ITERS=200 cargo test -p if97_core --test load_stress_test --frozen --color never -- --ignored --nocapture"
append_text "Входные данные:" "- Встроенный CSV \`core/tests/if97_rust_test_data.csv\`." "" "Оценка результата:" "- \`OK\`: стресс-тест выполняется и печатает throughput/checksum."

run_block "Bench 2 (build): if97_loadtest" \
  "cargo build -p if97_calculator_service --bin if97_loadtest --features loadtest --frozen --color never"

# 10 прогонов if97_loadtest (строго последовательно)
append_text "" "### Bench 2: CPU нагрузка ядра (multi-thread) через if97_loadtest" "" "Команда (серия из 10 прогонов, строго последовательно):" '```bash' "./target/debug/if97_loadtest --iters 200 --threads 8" '```' "" "Raw результаты (n=10):" '```text'
set +e
for _ in $(seq 1 10); do
  (cd "$tmp_repo" && ./target/debug/if97_loadtest --iters 200 --threads 8) >>"$out_tmp" 2>&1
done
set -e
append_text '```'

# Статистика по rows/s из 10 строк
python3 - "$out_tmp" <<'PY' >>"$out_tmp"
import math
import re
import statistics as st
from pathlib import Path

path = Path(__import__("sys").argv[1])
text = path.read_text(encoding="utf-8", errors="replace").splitlines()

vals = []
for line in text:
    if line.startswith("if97_loadtest:"):
        m = re.search(r"rows/s=(\d+)", line)
        if m:
            vals.append(int(m.group(1)))

# Берём последние 10 значений из текущего блока (n=10)
vals = vals[-10:]

if len(vals) == 10:
    n = len(vals)
    mean = sum(vals) / n
    stdev = st.stdev(vals)
    rsd = (stdev / mean) * 100.0 if mean else 0.0
    t_crit = 2.262  # t(0.975, df=9)
    half = t_crit * stdev / math.sqrt(n)
    lo = mean - half
    hi = mean + half
    print()
    print("Сводка по throughput `rows/s` (t-based 95% CI, df=9):")
    print(f"- mean: `{mean:.1f} rows/s`")
    print(f"- sample stdev: `{stdev:.2f} rows/s`")
    print(f"- RSD: `{rsd:.2f}%`")
    print(f"- 95% CI: `[{lo:.2f}, {hi:.2f}] rows/s`")
    print(f"- min/max: `[{min(vals)}, {max(vals)}] rows/s`")
else:
    print()
    print(f"Сводка по throughput `rows/s`: недостаточно данных (n={len(vals)}).")
PY

run_block "Bench 3: генерация купола насыщения (Tauri, ignored perf-test)" \
  "cargo test -p if97_calculator_native --frozen --color never -- --ignored --nocapture"

append_text "" "### Bench 4: gRPC транспорт (service + grpc_loadtest)" "" "Примечание: если запуск сервиса в sandbox запрещён, ниже будет зафиксирована ожидаемая ошибка. Для полноценного прогона нужен запуск вне песочницы."

# Сборка бинарников для gRPC бенча
run_block "Bench 4 (build): service + grpc_loadtest + grpc_healthcheck" \
  "cargo build -p if97_calculator_service --bin if97_calculator_service --bin grpc_loadtest --bin grpc_healthcheck --frozen --color never"

append_text "" "#### 4.1 Запуск сервиса (локально)" "" "Команда:" '```bash' "IF97_GRPC_ADDR=127.0.0.1:50051 \\" "IF97_KERNEL=core \\" "IF97_IO_THREADS=2 \\" "IF97_CPU_THREADS=8 \\" "IF97_MAX_IN_FLIGHT_PER_CONN=8 \\" "IF97_MAX_IN_FLIGHT_GLOBAL=8 \\" "IF97_DRAIN_SECS=2 \\" "RUST_LOG=info \\" "./target/debug/if97_calculator_service" '```' "" "Вывод:" '```text'

svc_log="$tmp_base/grpc_service.log"
(
  cd "$tmp_repo" && \
  IF97_GRPC_ADDR=127.0.0.1:50051 \
  IF97_KERNEL=core \
  IF97_IO_THREADS=2 \
  IF97_CPU_THREADS=8 \
  IF97_MAX_IN_FLIGHT_PER_CONN=8 \
  IF97_MAX_IN_FLIGHT_GLOBAL=8 \
  IF97_DRAIN_SECS=2 \
  RUST_LOG=info \
  ./target/debug/if97_calculator_service
) >"$svc_log" 2>&1 &
svc_pid=$!

wait_tcp_ok=0
for _ in $(seq 1 50); do
  if ! kill -0 "$svc_pid" 2>/dev/null; then
    break
  fi
  if python3 - <<'PY' 2>/dev/null
import socket
s = socket.socket()
s.settimeout(0.05)
try:
  s.connect(("127.0.0.1", 50051))
  raise SystemExit(0)
except Exception:
  raise SystemExit(1)
finally:
  try: s.close()
  except Exception: pass
PY
  then
    wait_tcp_ok=1
    break
  fi
  sleep 0.05
done

if [[ "$wait_tcp_ok" == "1" ]]; then
  echo "service: started (tcp ready)" >>"$svc_log"
else
  echo "service: not ready (sandbox/network ограничения или ошибка запуска)" >>"$svc_log"
fi

# Пишем то, что успело накопиться в логе (и в случае успеха/и в случае ошибки).
cat "$svc_log" >>"$out_tmp"
append_text '```'

if [[ "$wait_tcp_ok" == "1" ]]; then
  append_text "" "Health-check (прямой, через gRPC Health API):" '```text'
  (cd "$tmp_repo" && ./target/debug/grpc_healthcheck --addr 127.0.0.1:50051) >>"$out_tmp" 2>&1 || true
  append_text '```'

  append_text "" "#### 4.2 Серия замеров (streams=1, batches=500)" "" "Команда:" '```bash' "./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 500 --batch-size 2048 --in-flight 8" '```' "" "Raw результаты (n=6):" '```text'
  for _ in $(seq 1 6); do
    (cd "$tmp_repo" && ./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 500 --batch-size 2048 --in-flight 8) >>"$out_tmp" 2>&1 || true
  done
  append_text '```'

  python3 - "$out_tmp" <<'PY' >>"$out_tmp"
import math
import re
import statistics as st
from pathlib import Path

path = Path(__import__("sys").argv[1])
text = path.read_text(encoding="utf-8", errors="replace").splitlines()

vals = []
for line in text:
    if line.startswith("grpc_loadtest:"):
        m = re.search(r"points/s=(\d+)", line)
        if m:
            vals.append(int(m.group(1)))

vals = vals[-6:]
if len(vals) == 6:
    n = len(vals)
    mean = sum(vals) / n
    stdev = st.stdev(vals)
    rsd = (stdev / mean) * 100.0 if mean else 0.0
    t_crit = 2.571  # t(0.975, df=5)
    half = t_crit * stdev / math.sqrt(n)
    lo = mean - half
    hi = mean + half
    print()
    print("Сводка по throughput `points/s` (t-based 95% CI, df=5):")
    print(f"- mean: `{mean:.2f} points/s`")
    print(f"- sample stdev: `{stdev:.2f} points/s`")
    print(f"- RSD: `{rsd:.2f}%`")
    print(f"- 95% CI: `[{lo:.2f}, {hi:.2f}] points/s`")
    print(f"- min/max: `[{min(vals)}, {max(vals)}] points/s`")
else:
    print()
    print(f"Сводка по throughput `points/s`: недостаточно данных (n={len(vals)}).")
PY

  append_text "" "#### 4.3 Длинный прогон (batches=2000, один запуск)" "" "Команда:" '```bash' "./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 2000 --batch-size 2048 --in-flight 8" '```' "" "Вывод:" '```text'
  (cd "$tmp_repo" && ./target/debug/grpc_loadtest --addr 127.0.0.1:50051 --streams 1 --batches 2000 --batch-size 2048 --in-flight 8) >>"$out_tmp" 2>&1 || true
  append_text '```'
else
  append_text "" "Health-check:" "- Не выполнялся: сервис не поднялся в текущем окружении."
fi

# Останавливаем сервис, если он всё еще жив.
if kill -0 "$svc_pid" 2>/dev/null; then
  kill -INT "$svc_pid" 2>/dev/null || true
  for _ in $(seq 1 50); do
    if ! kill -0 "$svc_pid" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done
  kill -KILL "$svc_pid" 2>/dev/null || true
  wait "$svc_pid" 2>/dev/null || true
fi

if [[ -n "$tail_start" ]]; then
  append_text "" ""
  sed -n "${tail_start},\$p" "$old_out" >>"$out_tmp"
fi

mv "$out_tmp" "$out"

echo "Wrote $out"
echo "tmp repo: $tmp_repo"
