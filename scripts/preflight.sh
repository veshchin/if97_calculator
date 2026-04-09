#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

echo "repo: $root"
echo "date: $(date -Iseconds)"

echo
echo "rustc: $(rustc --version 2>/dev/null || echo MISSING)"
echo "cargo: $(cargo --version 2>/dev/null || echo MISSING)"
echo "rustup: $(rustup --version 2>/dev/null || echo MISSING)"

echo
echo "targets installed:"
rustup target list --installed 2>/dev/null || true

echo
echo "protoc: $(protoc --version 2>/dev/null || echo MISSING)"
echo "trunk: $(trunk --version 2>/dev/null || echo MISSING)"
echo "cargo-tauri: $(cargo tauri --version 2>/dev/null || echo MISSING)"

if [[ -n "${NO_COLOR-}" ]]; then
  case "${NO_COLOR}" in
    true|false) ;;
    *)
      echo
      echo "WARN: NO_COLOR=${NO_COLOR} может ломать trunk (trunk ожидает true|false)."
      echo "      Используйте: NO_COLOR=true trunk build --release"
      ;;
  esac
fi

echo
echo "note:"
echo "- crate fltk использует feature fltk-bundled; на первом билде может потребоваться сеть для скачивания bundled libs."

