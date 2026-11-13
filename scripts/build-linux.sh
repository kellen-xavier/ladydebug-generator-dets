#!/usr/bin/env bash
set -euo pipefail
raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$raiz"
cargo build --release --bin det-api
(cd app && flutter build linux --release)
cp target/release/det-api app/build/linux/x64/release/bundle/det-api
printf 'Pacote Linux: %s\n' "$raiz/app/build/linux/x64/release/bundle"
