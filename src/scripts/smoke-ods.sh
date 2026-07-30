#!/usr/bin/env bash
# Legacy entrypoint — prefer smoke-odc.sh
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${DIR}/smoke-odc.sh" "$@"
