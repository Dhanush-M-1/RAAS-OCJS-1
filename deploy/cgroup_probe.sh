#!/bin/bash
###############################################################################
# Prove the judge's cgroup mechanism works on this host.
#
# This does not read a config file and declare victory. It starts a real
# container under the same limits the judge uses, finds its cgroup the same way
# the judge does, and then verifies every file the judge reads or writes is
# present and usable. Finally it makes the container exceed memory.high and
# checks that the kernel actually incremented the pressure counter, because that
# counter increment is the event reactive promotion waits for.
#
# Run as root:  sudo ./cgroup_probe.sh
# Exits non-zero if any check fails.
###############################################################################
set -uo pipefail

CONTAINER_NAME="raas-cgroup-probe-$$"
CGROUP_FS_ROOT=/sys/fs/cgroup
IMAGE="${PROBE_IMAGE:-python-judge-runtime}"
LOW_TIER_MB="${LOW_TIER_MB:-256}"
WATERMARK_BYTES=$(( LOW_TIER_MB * 1024 * 1024 * 70 / 100 ))   # 70% watermark, as in docker.rs

FAILURES=0
pass() { printf '  \033[32mPASS\033[0m  %s\n' "$*"; }
fail() { printf '  \033[31mFAIL\033[0m  %s\n' "$*"; FAILURES=$((FAILURES + 1)); }
info() { printf '        %s\n' "$*"; }
head_() { printf '\n\033[1m%s\033[0m\n' "$*"; }

cleanup() {
  docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
}
trap cleanup EXIT

[ "$(id -u)" -eq 0 ] || { echo "must run as root (sudo $0)"; exit 2; }

head_ "1. cgroup hierarchy"
CG=$(stat -fc %T $CGROUP_FS_ROOT)
info "filesystem type: $CG"
if [ "$CG" = "cgroup2fs" ]; then
  pass "cgroup v2 unified hierarchy in use"
else
  fail "expected cgroup2fs (v2 unified), got '$CG'. memory.high does not exist on cgroup v1."
  echo "Cannot continue; the judge's promotion mechanism cannot work here."
  exit 1
fi

head_ "2. Docker cgroup driver"
DRIVER=$(docker info --format '{{.CgroupDriver}}' 2>/dev/null || echo unknown)
info "driver: $DRIVER"
if [ "$DRIVER" = "systemd" ]; then
  pass "Docker is using the systemd cgroup driver (pinned in /etc/docker/daemon.json)"
else
  # Not fatal: the judge reads the real mount path from /proc/<pid>/cgroup, so it
  # resolves correctly under cgroupfs too. Flagged because the cgroupfs driver
  # alongside systemd is a fragile combination on a booted systemd host.
  printf '  \033[33mWARN\033[0m  driver is %s, not systemd. The judge should still resolve\n' "$DRIVER"
  printf '        the path correctly, but this is not the tested configuration.\n'
fi

head_ "3. Starting a container under the judge's Low-tier limits"
info "image: $IMAGE, --memory=${LOW_TIER_MB}m --memory-swap=${LOW_TIER_MB}m --cpus=1"
if ! docker run -d --name "$CONTAINER_NAME" --network=none \
      --memory="${LOW_TIER_MB}m" --memory-swap="${LOW_TIER_MB}m" --cpus=1 \
      "$IMAGE" sleep infinity >/dev/null 2>&1; then
  fail "could not start a probe container"
  exit 1
fi
sleep 2
pass "container started"

head_ "4. Locating the cgroup the same way moderator.rs does"
PID=$(docker inspect --format '{{.State.Pid}}' "$CONTAINER_NAME" 2>/dev/null)
info "container init pid: $PID"
[ -n "$PID" ] && [ "$PID" != "0" ] || { fail "no container pid"; exit 1; }

PROC_CGROUP=$(cat "/proc/$PID/cgroup" 2>/dev/null)
info "/proc/$PID/cgroup -> $PROC_CGROUP"

REL=$(printf '%s\n' "$PROC_CGROUP" | awk -F'::' '/::/ {print $2; exit}')
info "mount-relative path: $REL"

CG_DIR=""
if [ -n "$REL" ]; then
  CAND="$CGROUP_FS_ROOT${REL}"
  info "candidate: $CAND"
  if [ -e "$CAND/memory.events" ]; then
    CG_DIR="$CAND"
  fi
fi

# Mirror the judge's fallback chain if the primary route did not resolve.
if [ -z "$CG_DIR" ]; then
  FULL_ID=$(docker inspect --format '{{.Id}}' "$CONTAINER_NAME" 2>/dev/null)
  info "primary route failed; trying fallback layouts for id $FULL_ID"
  for c in "$CGROUP_FS_ROOT/system.slice/docker-$FULL_ID.scope" \
           "$CGROUP_FS_ROOT/docker-$FULL_ID.scope" \
           "$CGROUP_FS_ROOT/docker/$FULL_ID" \
           "$CGROUP_FS_ROOT/$FULL_ID"; do
    if [ -e "$c/memory.events" ] || [ -e "$c/memory.current" ]; then CG_DIR="$c"; break; fi
  done
fi

if [ -n "$CG_DIR" ]; then
  pass "resolved container cgroup: $CG_DIR"
else
  fail "could not resolve the container cgroup by any route"
  echo "This is the failure that breaks promotion outright."
  exit 1
fi

head_ "5. Files the judge depends on"
for f in memory.high memory.max memory.current memory.events; do
  if [ -r "$CG_DIR/$f" ]; then
    pass "$f is readable  (current: $(tr -d '\n' < "$CG_DIR/$f" | head -c 60))"
  else
    fail "$f is missing or unreadable - the judge cannot use this cgroup"
  fi
done

if grep -q '^high ' "$CG_DIR/memory.events" 2>/dev/null; then
  pass "memory.events contains a 'high' counter (the promotion trigger)"
  info "counters: $(tr '\n' ' ' < "$CG_DIR/memory.events")"
else
  fail "memory.events has no 'high' counter; reactive promotion has nothing to observe"
fi

head_ "6. Writing memory.high (the exact operation that fails without root)"
ORIG=$(tr -d '\n' < "$CG_DIR/memory.high" 2>/dev/null)
info "before: $ORIG"
TARGET=$(( LOW_TIER_MB * 1024 * 1024 / 2 ))
if echo "$TARGET" > "$CG_DIR/memory.high" 2>/dev/null; then
  AFTER=$(tr -d '\n' < "$CG_DIR/memory.high")
  info "wrote $TARGET, read back: $AFTER"
  if [ "$AFTER" = "$TARGET" ]; then
    pass "memory.high is writable and the value took effect"
  else
    fail "wrote $TARGET but read back '$AFTER'"
  fi
else
  fail "could not write memory.high (EACCES would appear here for a non-root process)"
fi

head_ "7. Does the kernel actually raise the pressure counter?"
# Set a small soft limit, then let the container exceed it. If the 'high'
# counter increments, the event reactive promotion watches for is real.
echo $(( 32 * 1024 * 1024 )) > "$CG_DIR/memory.high" 2>/dev/null
BEFORE_HIGH=$(awk '$1=="high"{print $2}' "$CG_DIR/memory.events" 2>/dev/null || echo 0)
info "soft limit set to 32 MiB; 'high' counter before: ${BEFORE_HIGH:-0}"
info "allocating ~96 MiB inside the container and touching every page..."

docker exec "$CONTAINER_NAME" python3 -c "
b = bytearray(96*1024*1024)
for i in range(0, len(b), 4096):
    b[i] = 1
print('allocated')
" >/dev/null 2>&1 &
EXEC_PID=$!
# memory.high throttles rather than killing, so this can be slow by design.
for _ in $(seq 1 60); do
  sleep 1
  NOW=$(awk '$1=="high"{print $2}' "$CG_DIR/memory.events" 2>/dev/null || echo 0)
  [ "${NOW:-0}" -gt "${BEFORE_HIGH:-0}" ] && break
done
kill "$EXEC_PID" 2>/dev/null || true

AFTER_HIGH=$(awk '$1=="high"{print $2}' "$CG_DIR/memory.events" 2>/dev/null || echo 0)
info "'high' counter after: ${AFTER_HIGH:-0}"
if [ "${AFTER_HIGH:-0}" -gt "${BEFORE_HIGH:-0}" ]; then
  pass "kernel incremented the 'high' counter - the promotion trigger is real on this host"
else
  fail "'high' counter did not move. The judge would arm memory.high but never see a promotion event."
  info "This is the silent-failure mode. Do not trust benchmark numbers from this host."
fi

head_ "Summary"
if [ "$FAILURES" -eq 0 ]; then
  printf '\033[32m  All checks passed.\033[0m The judge'"'"'s cgroup mechanism works on this host.\n'
  exit 0
else
  printf '\033[31m  %d check(s) failed.\033[0m Promotion will not work reliably.\n' "$FAILURES"
  exit 1
fi
