#!/usr/bin/env bash
set -euo pipefail

mode="${1:-html}"

case "$mode" in
  summary)
    cargo llvm-cov --workspace --summary-only
    ;;
  html)
    cargo llvm-cov --workspace --html --output-dir target/llvm-cov/html
    printf 'HTML coverage report: target/llvm-cov/html/index.html\n'
    ;;
  lcov)
    mkdir -p target/llvm-cov
    cargo llvm-cov --workspace --lcov --output-path target/llvm-cov/lcov.info
    printf 'LCOV report: target/llvm-cov/lcov.info\n'
    ;;
  all)
    mkdir -p target/llvm-cov
    cargo llvm-cov --workspace --summary-only
    cargo llvm-cov --workspace --html --output-dir target/llvm-cov/html
    cargo llvm-cov --workspace --lcov --output-path target/llvm-cov/lcov.info
    printf 'HTML coverage report: target/llvm-cov/html/index.html\n'
    printf 'LCOV report: target/llvm-cov/lcov.info\n'
    ;;
  *)
    printf 'usage: %s [summary|html|lcov|all]\n' "$0" >&2
    exit 1
    ;;
esac
