#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/debug/build_bin.sh <bin> <mode>

Arguments:
  <bin>   : client | server | architecture_metrics
  <mode>  : fast | deep

Examples:
  scripts/debug/build_bin.sh client fast
  scripts/debug/build_bin.sh server deep
EOF
}

if [[ $# -ne 2 ]]; then
  usage
  exit 2
fi

bin="$1"
mode="$2"

case "$bin" in
  client|server|architecture_metrics) ;;
  *)
    echo "error: invalid bin '$bin'" >&2
    usage
    exit 2
    ;;
esac

case "$mode" in
  fast)
    target_dir="target-linux"
    profile_dir="debug"
    profile_args=()
    ;;
  deep)
    target_dir="target-linux-devdebug"
    profile_dir="dev-debug"
    profile_args=(--profile dev-debug)
    ;;
  *)
    echo "error: invalid mode '$mode'" >&2
    usage
    exit 2
    ;;
esac

for cmd in cargo ss nc; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 1
  fi
done

echo "[debug-build] bin=$bin mode=$mode target_dir=$target_dir profile_dir=$profile_dir"

if [[ "$bin" == "server" ]]; then
  if ss -H -ltn "sport = :8080" | grep -q .; then
    echo "[preflight][warn] port 8080 already in use:"
    ss -ltnp "sport = :8080" || true
  else
    echo "[preflight] port 8080 is available"
  fi

  declare -a service_ports=(
    "5432:PostgreSQL"
    "6379:Redis"
    "5672:RabbitMQ"
  )
  for item in "${service_ports[@]}"; do
    port="${item%%:*}"
    name="${item#*:}"
    if nc -z -w1 127.0.0.1 "$port" >/dev/null 2>&1; then
      echo "[preflight] service reachable: $name (127.0.0.1:$port)"
    else
      echo "[preflight][warn] service unreachable: $name (127.0.0.1:$port)"
    fi
  done
elif [[ "$bin" == "client" ]]; then
  if nc -z -w1 127.0.0.1 8080 >/dev/null 2>&1; then
    echo "[preflight] ws endpoint reachable: 127.0.0.1:8080"
  else
    echo "[preflight][warn] ws endpoint unreachable: 127.0.0.1:8080"
  fi
fi

cargo_args=(
  build
  --package emiyashiro
  --bin "$bin"
  --target-dir "$target_dir"
  "${profile_args[@]}"
)

if [[ "$bin" == "server" ]]; then
  cargo_args+=(--no-default-features --features server)
fi

echo "[debug-build] cargo ${cargo_args[*]}"
cargo "${cargo_args[@]}"

artifact="$target_dir/$profile_dir/$bin"
if [[ ! -x "$artifact" ]]; then
  echo "error: expected executable not found: $artifact" >&2
  exit 1
fi

echo "[debug-build] ready: $artifact"
