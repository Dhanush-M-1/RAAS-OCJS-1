#!/usr/bin/env bash
###############################################################################
# Start the judge on a freshly provisioned VM and prove it actually works.
#
# Terraform deliberately stops short of running the judge. It installs and
# enables the unit but leaves it stopped, so the service can never come up
# unauthenticated. This script supplies the shared secret, starts it, and then
# verifies the two things that decide whether the deployment is worth anything:
#
#   1. /submit rejects a request with no token.
#   2. A submission that crosses the 179.2 MiB watermark is promoted.
#
# If either check fails the script exits non-zero rather than reporting success.
###############################################################################
set -euo pipefail

TF_DIR="${TF_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/terraform" && pwd)}"
ENV_FILE=/etc/raas/judge.env
TOKEN_HEADER=x-raas-token

say()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
fail() { printf '\n\033[1;31mFAIL: %s\033[0m\n' "$*" >&2; exit 1; }

###############################################################################
# Resolve the target from Terraform state, so there is one source of truth.
###############################################################################
say "Reading Terraform outputs"
INSTANCE=$(terraform -chdir="$TF_DIR" output -raw instance_name)
ZONE=$(terraform -chdir="$TF_DIR" output -raw zone)
PROJECT=$(terraform -chdir="$TF_DIR" output -raw project_id)

echo "instance : $INSTANCE"
echo "zone     : $ZONE"
echo "project  : $PROJECT"

rssh() {
  gcloud compute ssh "$INSTANCE" --zone="$ZONE" --project="$PROJECT" \
    --tunnel-through-iap --quiet --command="$1"
}

###############################################################################
# 1. Wait for the first-boot bootstrap to finish.
###############################################################################
say "Waiting for first-boot bootstrap (Docker, images, Rust build)"
DEADLINE=$((SECONDS + 2400))
while :; do
  if rssh 'grep -q "bootstrap complete" /var/log/raas-bootstrap.log 2>/dev/null' 2>/dev/null; then
    echo "bootstrap reported complete"
    break
  fi
  if rssh 'grep -q "\[FATAL\]" /var/log/raas-bootstrap.log 2>/dev/null' 2>/dev/null; then
    rssh 'grep "\[FATAL\]" /var/log/raas-bootstrap.log' || true
    fail "bootstrap aborted on the VM (see FATAL lines above)"
  fi
  [ "$SECONDS" -lt "$DEADLINE" ] || fail "bootstrap did not finish within 40 minutes"
  printf '.'
  sleep 20
done

###############################################################################
# 2. Preflight. The cgroup driver is the one that fails silently, so it is
#    checked explicitly rather than assumed from the config file.
###############################################################################
say "Preflight: cgroup driver and runtime images"
# Docker commands need root: the SSH user is not in the docker group, and only the
# bootstrap (running as root) could reach the socket. GCE grants the SSH user
# passwordless sudo, so this is safe to do directly.
DRIVER=$(rssh 'sudo docker info --format "{{.CgroupDriver}}"')
echo "cgroup driver   : $DRIVER"
if [ "$DRIVER" != "systemd" ]; then
  printf '\033[33mWARN\033[0m  Docker cgroup driver is %s, not systemd.\n' "$DRIVER"
  printf '      Not fatal: moderator.rs reads the real mount-relative path out of\n'
  printf '      /proc/<pid>/cgroup, so it resolves under either layout. Still worth\n'
  printf '      fixing, since cgroupfs beside systemd is a fragile pairing.\n'
fi

CG=$(rssh 'stat -fc %T /sys/fs/cgroup')
echo "cgroup hierarchy: $CG"
[ "$CG" = "cgroup2fs" ] || fail "expected cgroup v2 (cgroup2fs), got '$CG' - memory.high does not exist on v1"

IMAGES=$(rssh 'sudo docker images --format "{{.Repository}}" | grep -c "judge-runtime" || true')
echo "runtime images  : $IMAGES"
[ "$IMAGES" -ge 3 ] || fail "expected 3 judge-runtime images, found $IMAGES"

###############################################################################
# 2b. Exercise the cgroup mechanism end to end against a real container.
#     Reading the daemon's configured driver is not evidence that promotion
#     works; this writes memory.high and confirms the kernel raises the pressure
#     counter. Everything downstream depends on it, so it runs before any
#     benchmark time is spent.
###############################################################################
say "Proving the cgroup mechanism against a real container"
if ! rssh 'sudo bash /opt/raas/deploy/cgroup_probe.sh'; then
  fail "the cgroup probe failed on the VM. Promotion will not work, so any
benchmark run would produce plausible but wrong numbers. See the probe output
above for which check failed."
fi

###############################################################################
# 3. Shared secret. Generated here, never by Terraform, so it stays out of
#    terraform.tfstate. Written with 0600 and never echoed to the terminal.
###############################################################################
say "Installing shared secret"
TOKEN=$(openssl rand -hex 32)
printf 'RAAS_AUTH_TOKEN=%s\n' "$TOKEN" \
  | rssh "sudo install -m 0600 /dev/stdin $ENV_FILE && echo wrote-$ENV_FILE"
echo "token written to $ENV_FILE (64 hex chars, not printed)"

###############################################################################
# 4. Start and confirm it stayed up.
###############################################################################
say "Starting the judge"
rssh 'sudo systemctl restart raas-judge'
sleep 5
STATE=$(rssh 'systemctl is-active raas-judge || true')
echo "unit state      : $STATE"
[ "$STATE" = "active" ] || { rssh 'sudo journalctl -u raas-judge -n 40 --no-pager' || true; fail "unit is '$STATE'"; }

HEALTH=$(rssh 'curl -s --max-time 10 localhost:3000/health')
echo "health          : $HEALTH"
echo "$HEALTH" | grep -q '"OK"' || fail "/health did not return OK"

###############################################################################
# 5. Auth must actually reject. A judge that answers an unauthenticated request
#    is the exact condition this deployment exists to avoid.
###############################################################################
say "Verifying /submit rejects an unauthenticated request"
UNAUTH=$(rssh "curl -s -o /dev/null -w '%{http_code}' --max-time 10 -X POST localhost:3000/submit -H 'content-type: application/json' -d '{}'")
echo "no token       : HTTP $UNAUTH"
[ "$UNAUTH" = "401" ] || fail "unauthenticated /submit returned $UNAUTH, expected 401"

AUTH=$(rssh "curl -s -o /dev/null -w '%{http_code}' --max-time 10 -X POST localhost:3000/submit -H 'content-type: application/json' -H '$TOKEN_HEADER: $TOKEN' -d '{}'")
echo "with token     : HTTP $AUTH"
[ "$AUTH" != "401" ] || fail "a valid token was rejected"
[ "$AUTH" != "405" ] || fail "/submit is not accepting POST"

###############################################################################
# 6. The test that decides whether any of this is worth benchmarking: a program
#    that crosses the 179.2 MiB watermark must be promoted from Low to High.
###############################################################################
say "Promotion smoke test (a ~200 MiB allocation must be promoted)"

HEAVY_C=$(cat <<'SRC'
#include <stdio.h>
#include <stdlib.h>
int main(void){
    size_t n = 200UL*1024*1024;
    char *p = malloc(n);
    if(!p){ printf("0\n"); return 1; }
    for(size_t i=0;i<n;i+=4096) p[i]=1;
    printf("%d\n", p[0]==p[0] ? 1 : 0);
    free(p);
    return 0;
}
SRC
)

rssh 'true' # establish the IAP tunnel and SSH key before the real work

TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT
printf '%s' "$HEAVY_C" > "$TMPD/heavy.c"
python3 -c '
import json, sys
src = open(sys.argv[1]).read()
print(json.dumps({
    "id": "smoke-heavy-1",
    "language": "c",
    "source": src,
    "test_cases": [{"input": "1", "expected": "1"}],
    "approach": "reactive",
}))
' "$TMPD/heavy.c" > "$TMPD/heavy.json"
echo "payload         : $(wc -c < "$TMPD/heavy.json") bytes"

# Base64 rather than stdin: keeps the payload intact through the IAP tunnel.
B64=$(base64 -w0 "$TMPD/heavy.json")
PROMO=$(rssh "echo '$B64' | base64 -d > /tmp/heavy.json && curl -s --max-time 300 -X POST localhost:3000/submit -H 'content-type: application/json' -H '$TOKEN_HEADER: $TOKEN' --data-binary @/tmp/heavy.json")

echo "$PROMO" | python3 -m json.tool 2>/dev/null || echo "$PROMO"

PROMOTED=$(echo "$PROMO" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("tier_promoted"))' 2>/dev/null || echo parse-error)
STARTED=$(echo "$PROMO" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("tier_started"))' 2>/dev/null || echo parse-error)
PEAK=$(echo "$PROMO" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("peak_memory_bytes"))' 2>/dev/null || echo 0)

echo "tier started    : $STARTED"
echo "tier promoted   : $PROMOTED"
echo "peak memory     : $PEAK bytes"

printf '\n\033[1mSummary\033[0m\n'
printf '  cgroup driver   : %s (must be systemd)\n' "$DRIVER"
printf '  cgroup v2       : %s\n' "$CG"
printf '  auth enforced   : yes (401 without token)\n'
printf '  promotion works : %s\n' "$PROMOTED"

if [ "$PROMOTED" != "True" ]; then
  fail "the judge did NOT promote the ~200 MiB submission.

This is the failure the whole deployment hinges on. Most likely causes:
  - Docker is not using the systemd cgroup driver, so the /proc/<pid>/cgroup
    lookup misses and memory.high is never armed.
  - The unit is not running as root, so writing memory.high returned EACCES.
  - The watermark (179.2 MiB at LOW_TIER_MB=256) was not actually crossed.

Check with:
  $ZONE / $INSTANCE: sudo journalctl -u raas-judge -n 80 --no-pager"
fi

say "Deployment verified. The judge is running and promoting."
