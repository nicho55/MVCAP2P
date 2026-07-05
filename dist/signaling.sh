#!/usr/bin/env bash
# Launcher do servidor de sinalização. Uma instância só, em qualquer uma das
# máquinas — as duas se conectam a ela pelo IP.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$here/signaling" "$@"
