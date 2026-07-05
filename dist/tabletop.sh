#!/usr/bin/env bash
# Launcher do Tabletop P2P. Roda de qualquer diretório: resolve o próprio caminho
# para achar o binário e a pasta assets/ ao lado dele.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$here"
exec "$here/tabletop" "$@"
