#!/usr/bin/env bash
set -euo pipefail

server_log="$(mktemp)"
client_log="$(mktemp)"
server_pid=""
client_pid=""

cleanup() {
  if [[ -n "$client_pid" ]]; then
    kill "$client_pid" 2>/dev/null || true
    wait "$client_pid" 2>/dev/null || true
  fi
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -f "$server_log" "$client_log"
}
trap cleanup EXIT

./target/debug/decentralized-ai-p2p >"$server_log" 2>&1 &
server_pid=$!

for _ in $(seq 1 50); do
  if grep -Eq '^listening_on=/ip4/(0\.0\.0\.0|127\.0\.0\.1)/tcp/' "$server_log"; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log"
    echo "server exited before listening" >&2
    exit 1
  fi
  sleep 0.2
done

peer_address="$(sed -n 's|^listening_on=/ip4/[^/]*/tcp/\([0-9]*\)$|/ip4/127.0.0.1/tcp/\1|p' "$server_log" | head -n 1)"
if [[ -z "$peer_address" ]]; then
  cat "$server_log"
  echo "server did not publish a TCP listen address" >&2
  exit 1
fi

./target/debug/decentralized-ai-p2p "$peer_address" --add 20 22 >"$client_log" 2>&1 &
client_pid=$!

for _ in $(seq 1 50); do
  if grep -q 'result=Some(42)' "$client_log"; then
    cat "$client_log"
    echo "P2P task request/response passed"
    exit 0
  fi
  if ! kill -0 "$client_pid" 2>/dev/null; then
    cat "$client_log"
    cat "$server_log"
    echo "client exited before receiving a task result" >&2
    exit 1
  fi
  sleep 0.2
done

cat "$client_log"
cat "$server_log"
echo "timed out waiting for task result" >&2
exit 1
