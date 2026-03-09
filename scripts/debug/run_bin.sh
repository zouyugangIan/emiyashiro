#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/debug/run_bin.sh <bin> <mode> [-- <args...>]

Arguments:
  <bin>   : client | server | architecture_metrics
  <mode>  : fast | deep

Examples:
  scripts/debug/run_bin.sh server fast
  scripts/debug/run_bin.sh client deep -- --my-arg value
EOF
}

if [[ $# -lt 2 ]]; then
  usage
  exit 2
fi

bin="$1"
mode="$2"
shift 2

extra_args=()
if [[ $# -gt 0 ]]; then
  if [[ "$1" == "--" ]]; then
    shift
  fi
  extra_args=("$@")
fi

case "$mode" in
  fast)
    target_dir="target-linux"
    profile_dir="debug"
    ;;
  deep)
    target_dir="target-linux-devdebug"
    profile_dir="dev-debug"
    ;;
  *)
    echo "error: invalid mode '$mode'" >&2
    usage
    exit 2
    ;;
esac

scripts/debug/build_bin.sh "$bin" "$mode"

artifact="$target_dir/$profile_dir/$bin"
if [[ ! -x "$artifact" ]]; then
  echo "error: executable not found: $artifact" >&2
  exit 1
fi

export RUST_BACKTRACE="${RUST_BACKTRACE:-full}"
export RUST_LIB_BACKTRACE="${RUST_LIB_BACKTRACE:-1}"

if [[ -z "${RUST_LOG:-}" ]]; then
  case "$bin" in
    server)
      export RUST_LOG="info,emiyashiro=debug,emiyashiro::systems::network=debug,sqlx=warn,redis=warn,lapin=warn"
      ;;
    client)
      export RUST_LOG="info,emiyashiro=debug,emiyashiro::systems::network=debug"
      ;;
    *)
      export RUST_LOG="info,emiyashiro=debug"
      ;;
  esac
fi

echo "[debug-run] exec $artifact ${extra_args[*]-}"
exec "$artifact" "${extra_args[@]}"
