#!/usr/bin/env bash
###############################################################################
# Re-run the VM bootstrap without recreating the VM.
#
# The startup script lives in the repository as a Terraform template, so it
# cannot simply be re-executed on the host: the ${...} placeholders are still
# unsubstituted there. This renders the template with the same values Terraform
# uses, uploads it, and runs it detached so an SSH timeout cannot kill a
# ten-minute build part-way through.
#
# Safe to re-run: every step checks whether its work is already done.
#
#   ./rerun_bootstrap.sh          # render, upload, run, then follow the log
#   ./rerun_bootstrap.sh --upload-only
###############################################################################
set -euo pipefail

TF_DIR="${TF_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/terraform" && pwd)}"
UPLOAD_ONLY=0
[ "${1:-}" = "--upload-only" ] && UPLOAD_ONLY=1

say() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

INSTANCE=$(terraform -chdir="$TF_DIR" output -raw instance_name)
ZONE=$(terraform -chdir="$TF_DIR" output -raw zone)
PROJECT=$(terraform -chdir="$TF_DIR" output -raw project_id)

# Values come from terraform.tfvars when present, else the variable defaults in
# variables.tf. Only these three are substituted into the template.
tfvar() {
  local key="$1" default="$2"
  if [ -f "$TF_DIR/terraform.tfvars" ]; then
    local v
    v=$(grep -E "^\s*${key}\s*=" "$TF_DIR/terraform.tfvars" | head -1 | sed 's/.*=\s*//' | tr -d '"' | tr -d ' ')
    [ -n "$v" ] && { echo "$v"; return; }
  fi
  echo "$default"
}

REPO_URL=$(tfvar repo_url "https://github.com/Hemanthkumar2k04/RAAS-OCJS.git")
REPO_REF=$(tfvar repo_ref "main")
LOW_TIER_MB=$(tfvar low_tier_mb "256")

say "Rendering startup.sh.tftpl"
echo "repo     : $REPO_URL"
echo "ref      : $REPO_REF"
echo "low tier : ${LOW_TIER_MB} MiB"

RENDERED=$(mktemp)
trap 'rm -f "$RENDERED"' EXIT

TEMPLATE="$TF_DIR/startup.sh.tftpl" \
REPO_URL="$REPO_URL" REPO_REF="$REPO_REF" LOW_TIER_MB="$LOW_TIER_MB" \
python3 - <<'PY' > "$RENDERED"
import os
tpl = open(os.environ["TEMPLATE"]).read()
tpl = tpl.replace("${repo_url}", os.environ["REPO_URL"])
tpl = tpl.replace("${repo_ref}", os.environ["REPO_REF"])
tpl = tpl.replace("${low_tier_mb}", os.environ["LOW_TIER_MB"])

# Check for the specific Terraform placeholders rather than any "${" occurrence:
# the script legitimately contains shell expansions such as "${HOME:-/root}",
# and flagging those would fail on a valid render.
leftover = [p for p in ("${repo_url}", "${repo_ref}", "${low_tier_mb}") if p in tpl]
assert not leftover, "unsubstituted placeholder(s): " + ", ".join(leftover)
print(tpl, end="")
PY

bash -n "$RENDERED" || { echo "rendered script failed syntax check"; exit 1; }
echo "rendered $(wc -l < "$RENDERED") lines, syntax OK"

say "Uploading to the VM"
B64=$(base64 -w0 "$RENDERED")
gcloud compute ssh "$INSTANCE" --zone="$ZONE" --project="$PROJECT" --tunnel-through-iap --quiet \
  --command="echo '$B64' | base64 -d | sudo tee /usr/local/bin/raas-bootstrap.sh > /dev/null && sudo chmod +x /usr/local/bin/raas-bootstrap.sh && echo uploaded"

[ "$UPLOAD_ONLY" -eq 1 ] && { echo "upload only; not running"; exit 0; }

say "Running the bootstrap detached (survives SSH disconnects)"
gcloud compute ssh "$INSTANCE" --zone="$ZONE" --project="$PROJECT" --tunnel-through-iap --quiet \
  --command="sudo truncate -s 0 /var/log/raas-bootstrap.log; sudo setsid nohup /usr/local/bin/raas-bootstrap.sh > /dev/null 2>&1 < /dev/null & echo started"

say "Following the log"
DEADLINE=$((SECONDS + 2100))
while :; do
  STATUS=$(gcloud compute ssh "$INSTANCE" --zone="$ZONE" --project="$PROJECT" --tunnel-through-iap --quiet \
    --command='if sudo grep -q "bootstrap complete" /var/log/raas-bootstrap.log 2>/dev/null; then echo DONE;
               elif sudo grep -q "\[FATAL\]" /var/log/raas-bootstrap.log 2>/dev/null; then echo FATAL;
               elif pgrep -f raas-bootstrap.sh >/dev/null; then echo RUNNING; else echo DEAD; fi' 2>/dev/null | tr -d '\r\n')
  case "$STATUS" in
    DONE)  echo; echo "bootstrap finished"; break ;;
    FATAL) echo; echo "bootstrap reported FATAL"; break ;;
    DEAD)  echo; echo "bootstrap process is gone without completing"; break ;;
    *)     printf '.'; sleep 20 ;;
  esac
  [ "$SECONDS" -lt "$DEADLINE" ] || { echo; echo "timed out waiting"; break; }
done

say "Bootstrap log tail"
gcloud compute ssh "$INSTANCE" --zone="$ZONE" --project="$PROJECT" --tunnel-through-iap --quiet \
  --command='sudo grep -E "^(===|\[verify\]|\[FATAL\]|All checks passed|[0-9]+ check)" /var/log/raas-bootstrap.log | tail -20; echo; sudo tail -6 /var/log/raas-bootstrap.log'
