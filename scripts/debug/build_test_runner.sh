#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/debug/build_test_runner.sh <mode>

Arguments:
  <mode> : fast | deep

Behavior:
  1) Builds lib test harness with --no-run.
  2) Creates a stable symlink used by Zed debugger:
     target-linux*/<profile>/emiyashiro-lib-tests
EOF
}

if [[ $# -ne 1 ]]; then
  usage
  exit 2
fi

mode="$1"
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

for cmd in cargo jq; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 1
  fi
done

echo "[debug-test] mode=$mode target_dir=$target_dir profile_dir=$profile_dir"

exe_path="$(
  cargo test \
    --package emiyashiro \
    --lib \
    --no-run \
    --target-dir "$target_dir" \
    "${profile_args[@]}" \
    --message-format=json \
    | jq -r '
        select(
          .reason == "compiler-artifact"
          and .profile.test == true
          and .executable != null
          and (.target.kind | index("lib"))
        ) | .executable
      ' \
    | tail -n 1
)"

if [[ -z "${exe_path:-}" ]]; then
  echo "error: failed to resolve lib test executable path from cargo output" >&2
  exit 1
fi

if [[ ! -x "$exe_path" ]]; then
  echo "error: resolved test executable is not runnable: $exe_path" >&2
  exit 1
fi

runner_link="$target_dir/$profile_dir/emiyashiro-lib-tests"
mkdir -p "$(dirname "$runner_link")"
ln -sfn "$(realpath "$exe_path")" "$runner_link"

echo "[debug-test] ready: $runner_link -> $(realpath "$exe_path")"
