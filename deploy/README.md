# Deploying the RAAS-OCJS judge to GCP

The judge runs on a single Compute Engine VM. This directory provisions that VM
with Terraform and then verifies the judge actually works.

## Why a VM and not a managed platform

The judge writes to the host cgroup tree as root and drives the Docker socket to
spawn sibling containers. Cloud Run, Cloud Functions, App Engine and GKE
Autopilot all withhold one or both of those, so none of them can host it. This is
a hard constraint from the code, not a preference:

| Requirement | Where it comes from |
| --- | --- |
| cgroup **v2**, unified hierarchy | `memory.high` does not exist under cgroup v1, so the judge has no watermark to arm |
| root, to write `memory.high` | `server/src/docker.rs`; unprivileged writes fail with `EACCES` and promotion never fires |
| the Docker socket | `server/src/docker.rs` shells out to `docker run/exec/update/rm` |
| three locally built images | `server/runtimes/{cpp,java,python}/Dockerfile`, published to no registry |

The Docker cgroup driver *is* pinned to `systemd` in `/etc/docker/daemon.json`, but
that is a stability measure rather than a hard requirement, and it is worth being
precise about why. `moderator.rs` resolves a container's cgroup by reading the real
mount-relative path out of `/proc/<pid>/cgroup` and joining it to `/sys/fs/cgroup`,
so it lands correctly under **both** the systemd layout
(`/system.slice/docker-<id>.scope`) and the cgroupfs layout (`/docker/<id>`). There
is a further fallback that tries four known layouts by container id. The pinning
matters because the cgroupfs driver beside a booted systemd host is a fragile
combination; it is not what makes promotion work.

## Security posture

`/submit` compiles and executes arbitrary submitted source. Three things stand
between that and the public internet:

1. **Firewall.** Ingress is default-deny. The only rule is IAP on `tcp/22`. Port
   3000 has no rule at all unless you explicitly add `judge_source_ranges`, so
   the default deployment is unreachable from the internet even though the
   service binds `0.0.0.0:3000`.
2. **Terraform never touches the secret.** `deploy.sh` generates the shared
   token and writes it to `/etc/raas/judge.env` on the VM. It is never passed
   through Terraform, so it cannot end up in `terraform.tfstate`.
3. **The judge service starts stopped.** Terraform installs and enables the unit
   but does not start it, so the judge is never briefly exposed unauthenticated
   while it waits for a secret.

`/submit` requires an `x-raas-token` header once `RAAS_AUTH_TOKEN` is set. A token
shorter than 16 characters is treated as unset, and the server logs a warning,
rather than creating false assurance.

## Prerequisites

```bash
gcloud auth login                       # gcloud CLI
gcloud auth application-default login   # Terraform's Google provider needs ADC
gcloud config set project raas-ocjs
```

The two are separate credential stores. `gcloud auth login` alone is not enough:
the Terraform provider looks for Application Default Credentials and fails with
"could not find default credentials" without the second command.

Terraform and gcloud are installed under `~/.local/bin` and `~/google-cloud-sdk`
on this machine, neither via sudo.

## Deploy

```bash
cd deploy/terraform
terraform init
terraform apply                     # provisions the VM, takes about a minute
cd ..
./deploy.sh                         # waits for bootstrap, then verifies
```

First boot installs Docker, builds the three runtime images, installs Rust and
compiles the judge. Expect roughly 10 to 15 minutes; `deploy.sh` polls the log and
reports progress rather than blocking silently.

`deploy.sh` finishes by running a submission that allocates ~200 MiB and asserts
that `tier_promoted` came back `true`. If that fails it exits non-zero, because a
judge that does not promote is not worth benchmarking.

## Reaching the judge

```bash
gcloud compute start-iap-tunnel raas-judge 3000 \
  --local-host-port=localhost:3000 --zone=asia-south1-a --project=raas-ocjs
```

Then talk to `http://localhost:3000`, adding the token header:

```bash
curl -s localhost:3000/health
curl -s -X POST localhost:3000/submit \
  -H 'content-type: application/json' \
  -H "x-raas-token: $(cat /path/to/token)" \
  --data @submission.json
```

The token lives at `/etc/raas/judge.env` on the VM:

```bash
gcloud compute ssh raas-judge --zone=asia-south1-a --project=raas-ocjs \
  --tunnel-through-iap --command='sudo cat /etc/raas/judge.env'
```

## Cost

`e2-standard-4` in `asia-south1` is about `$0.161/hr` ex-GST, approximately
`$0.19/hr` with India's 18% GST. Three hours is roughly **Rs 55**. A 30 GiB
`pd-balanced` boot disk adds about Rs 3.5/month.

The number that matters is the monthly one: **left running, this is roughly
Rs 13,400/month.**

## Tear down

```bash
cd deploy/terraform && terraform destroy
```

Stopping the VM is not enough. A stopped instance keeps billing for its boot
disk, so destroy it when the benchmark run is finished.

## Troubleshooting

**`memory.high` is never armed, promotion never fires.**
Run the probe, which tests this directly rather than inferring it from config:

```bash
sudo /opt/raas/deploy/cgroup_probe.sh
```

It resolves a container's cgroup exactly as `moderator.rs` does, writes
`memory.high`, and makes the container exceed it to confirm the kernel raises the
`high` pressure counter. If the counter does not move, promotion cannot fire, and
every latency or memory number from that host is meaningless. The probe exits
non-zero so it can gate a benchmark run.

Common causes, in the order the probe distinguishes them:

- **cgroup v1.** `stat -fc %T /sys/fs/cgroup` must print `cgroup2fs`.
- **Not running as root.** The unit deliberately has no `User=` directive; writing
  `memory.high` as a non-root process returns `EACCES`.
- **`memory.events` missing a `high` counter**, which means the memory controller
  is not enabled for that subtree.

**Docker cgroup driver is `cgroupfs`.** Not fatal. The judge reads the real path
from `/proc/<pid>/cgroup`, so it resolves correctly either way. It is still worth
fixing, because `cgroupfs` next to systemd is a fragile combination on a booted
host. Check with `docker info --format '{{.CgroupDriver}}'` and confirm
`/etc/docker/daemon.json` contains `"exec-opts": ["native.cgroupdriver=systemd"]`.

**`Permission denied (os error 13)` on `/sys/fs/cgroup`.** The unit is not
running as root. The service definition deliberately has no `User=` directive.

**Quota errors on apply.** New projects often cap E2 vCPUs below 4. Check
`gcloud compute regions describe asia-south1` for `CPUS_PER_VM_FAMILY` and request
an increase, or drop to `e2-standard-2` for a smoke test.

**`deploy.sh` cannot find the instance.** `terraform apply` must have succeeded
first; the script reads all of its targets from Terraform outputs.
