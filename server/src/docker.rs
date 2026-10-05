use crate::models::{CaseResult, Submission, TestCase};
use crate::moderator::{CGroup, locate_cgroup_dir};
use crate::policy::{MonitorSignal, Tier, TierPolicy};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{self, AsyncWriteExt};
use tokio::process::Command;

/// Hard memory limit (bytes) for a Low-tier container.
///
/// The shipped configuration is 256 MiB. `LOW_TIER_MB` exists so the tier can
/// be re-sized for the boundary experiments in Section VI without a code edit;
/// it is an experimental knob, and the default is the value we deploy.
///
/// NOTE: this must stay the *single* source of truth. The `--memory` flag
/// passed to `docker run` is derived from this value rather than hardcoded,
/// because previously the const and the flag were separate and could drift.
pub const LOW_MEM_HARD_LIMIT_DEFAULT_MB: u64 = 256;

/// Soft watermark as a percentage of the hard limit (tunable).
pub const HIGH_WATERMARK_PCT: u64 = 70;

/// Hard memory limit in bytes, resolved once from `LOW_TIER_MB` (default 256).
pub fn low_mem_hard_limit() -> u64 {
    static LIMIT: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *LIMIT.get_or_init(|| {
        let mb = std::env::var("LOW_TIER_MB")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(LOW_MEM_HARD_LIMIT_DEFAULT_MB);
        eprintln!("[config] LOW_TIER_MB: low tier hard limit = {mb} MiB");
        mb * 1024 * 1024
    })
}

/// Soft memory watermark (bytes) written to `memory.high` for a Low-tier start:
/// `HIGH_WATERMARK_PCT` of the hard limit (179.2 MiB at the default 256 MiB).
///
/// Docker's `--memory` sets `memory.max` (the hard OOM boundary) but does *not*
/// set `memory.high`. The kernel only counts `high` pressure events when
/// `memory.high` is configured, so the judge writes this watermark itself. It
/// sits below the hard limit, giving the reactive monitor a chance to promote a
/// heavy submission *before* it can be OOM-killed while allowing submissions
/// that fit within ~70% of the tier to complete without promotion.
pub fn low_mem_high_watermark() -> u64 {
    static WM: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *WM.get_or_init(|| low_mem_hard_limit() * HIGH_WATERMARK_PCT / 100)
}

/// How often the monitor re-reads the cgroup files while a test case runs.
/// Reading a cgroup file is sub-millisecond; `memory.events` counters are
/// monotonic, so reaction latency is bounded by this poll even if a spike is
/// shorter than the interval.
const MONITOR_POLL: Duration = Duration::from_millis(2);

/// Ceiling used when promotion has to go through `docker update` because the
/// host cgroup directory is not reachable (the Docker-Desktop-style fallback).
///
/// It must be an explicit finite value. `--memory 0` is **not** "unlimited" for
/// `docker update`: Docker reads 0 as "no change", re-reconciles the container's
/// cgroup against its configured limit, and silently discards the promotion.
/// The value only needs to exceed any submission we are willing to run; the
/// host's own memory is the real ceiling.
const PROMOTED_MEMORY_CEILING: &str = "8g";

fn image_for(language: &str) -> &'static str {
    match language {
        "python" => "python-judge-runtime",
        "java" => "java-judge-runtime",
        "c" | "cpp" | "c++" => "cpp-judge-runtime",
        _ => "python-judge-runtime",
    }
}

fn source_filename(language: &str) -> &'static str {
    match language {
        "python" => "main.py",
        "cpp" | "c++" => "main.cpp",
        "c" => "main.c",
        "java" => "Main.java",
        _ => "main.py",
    }
}

/// Docker container names must match `[a-zA-Z0-9][a-zA-Z0-9_.-]*`. A submission
/// id is client-supplied and may contain anything (e.g. `C++`, or a path
/// separator), which previously produced an invalid container name and an
/// opaque `SE` verdict. Map every disallowed byte to `_`, then collapse runs
/// of `_` so distinct ids stay distinct. The returned value always begins with
/// an alphanumeric character.
fn sanitize_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    // Collapse consecutive separators introduced above.
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_us = false;
    for ch in out.chars() {
        if ch == '_' && prev_us {
            continue;
        }
        prev_us = ch == '_';
        collapsed.push(ch);
    }
    // The first character must be alphanumeric, not `_`, `.` or `-`. Strip any
    // run of those, and fall back to a fixed name if nothing usable remains.
    let trimmed = collapsed.trim_start_matches(['_', '.', '-']);
    if trimmed.is_empty() {
        return "submission".to_string();
    }
    // If we rewrote anything, append a short hash of the original id. Two
    // different ids can sanitise to the same string (`x/y` and `x_y` both
    // become `x_y`), and colliding container names would let one submission's
    // `docker rm -f` tear down another's running container.
    if trimmed != id {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        id.hash(&mut h);
        return format!("{}_{:08x}", trimmed, h.finish() as u32);
    }
    trimmed.to_string()
}

fn write_source(submission: &Submission) -> std::io::Result<std::path::PathBuf> {
    let dir = std::env::temp_dir().join(format!("oj_{}", sanitize_id(&submission.id)));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir)?;
    std::fs::write(
        dir.join(source_filename(&submission.language)),
        &submission.source,
    )?;
    Ok(dir)
}

fn get_tier_limits(tier: &Tier) -> Vec<String> {
    match tier {
        // Low starts bounded (1 CPU / 256 MiB). `Tier::High` is unlimited today,
        // which is also the ceiling Reactive/Hybrid promote to (see moderator.rs).
        //
        // `--memory-swap` must be given explicitly. When `--memory` is set and
        // `--memory-swap` is omitted, Docker defaults the swap limit to the same
        // value as `--memory`, so the container may draw up to *twice* its
        // nominal size, half from RAM and half from swap. A container configured
        // for 128 MiB then completed with a 235 MB working set (measured: the
        // effective ceiling sits near 240 MB, not 128 MB), which silently
        // defeats the tier and inflates every peak reported for a capped run.
        //
        // Passing swap equal to memory leaves no swap allowance, so the limit
        // bounds resident memory instead of RAM plus swap.
        Tier::Low => {
            let mb = low_mem_hard_limit() / (1024 * 1024);
            vec![
                "--cpus=1".to_string(),
                format!("--memory={mb}m"),
                format!("--memory-swap={mb}m"),
            ]
        }
        _ => vec![],
    }
}

/// Share of the Low-tier hard limit handed to the JVM heap via `-Xmx`.
///
/// The rest is what a JVM needs *outside* the heap - metaspace, thread stacks,
/// code cache, GC structures - measured at roughly 40-60 MiB for these
/// submissions. Giving the whole tier to `-Xmx` lets the JVM grow into that
/// overhead and be OOM-killed anyway; 75% leaves ~64 MiB at the default 256 MiB
/// tier. This is the sizing the tier-floor experiment (TEST_PLAN E7) is expected to confirm.
pub const JAVA_HEAP_PCT_OF_LOW_TIER: u64 = 75;

/// Heap ceiling in MiB for a JVM submission, for the tier it is being started in.
///
/// The tier has to be the input here, not a constant: Low is a bounded container
/// and High is unbounded, so a heap sized for one is simply wrong in the other.
/// High keeps the historical 2x-low ceiling rather than no ceiling at all, so a
/// runaway submission cannot consume the whole host.
fn java_heap_mb(tier: &Tier) -> u64 {
    let low_mb = low_mem_hard_limit() / (1024 * 1024);
    match tier {
        Tier::Low => (low_mb * JAVA_HEAP_PCT_OF_LOW_TIER / 100).max(16),
        _ => low_mb * 2,
    }
}

/// What a full submission run produced, beyond the per-case verdicts.
pub struct RunOutcome {
    pub results: Vec<CaseResult>,
    pub tier_promoted: bool,
    pub promotion_time_ms: u64,
}

/// Wall-clock ceiling for a single test case.
///
/// Without this, an infinite loop holds its queue semaphore permit forever and
/// starves every other submission. A case exceeding the limit is killed and
/// graded `TLE`.
const CASE_TIMEOUT: Duration = Duration::from_secs(10);

/// The memory ceiling a case ran under, as reported to the client.
///
/// `0` means Uncapped, which is how the frontend renders it.
fn allocated_for(promoted: bool, tier: Tier) -> u64 {
    if promoted || tier == Tier::High {
        0
    } else {
        low_mem_hard_limit()
    }
}

/// Execute one test case inside the running container and stream its input.
/// Takes owned arguments so it can run on a `'static` spawned task while the
/// caller concurrently watches the container's cgroup.
async fn exec_case(args: Vec<String>, input: String) -> io::Result<std::process::Output> {
    let mut child = Command::new("docker")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("docker exec failed");
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).await?;
        drop(stdin); // close -> program sees EOF
    }
    child.wait_with_output().await
}

/// Run one test case while a monitor polls the container's cgroup.
///
/// The `docker exec` runs in a spawned task; meanwhile, on a ~2 ms tick we:
///   1. track the `memory.events` `high` counter delta, and if the policy is
///      watching and says promote, lift the container's memory limits to
///      `max` once (recording the promotion wall-clock time);
///   2. sample `memory.current` to approximate this case's peak (cgroup
///      `memory.peak` is not resettable in our environment, so we cannot
///      reset-and-read it per case).
#[allow(clippy::too_many_arguments)]
async fn run_case_monitored(
    container: &str,
    run_cmd: &[String],
    test: &TestCase,
    cg: Option<&CGroup>,
    watch: bool,
    policy: &(dyn TierPolicy + Send + Sync),
    tier: &Tier,
    started: Instant,
    promoted: &mut bool,
    promotion_time_ms: &mut u64,
) -> io::Result<CaseResult> {
    let case_start = Instant::now();
    let cpu_start_usec = cg.and_then(|c| c.cpu_usage_usec().ok());
    // `memory.events` counters are cumulative for the life of the cgroup, so
    // baseline the OOM counter now and look for an increase at the end of this
    // case. Without the baseline, an OOM on case 1 would mislabel every later
    // case in the same submission as MLE.
    let oom_kill_start = cg
        .and_then(|c| c.memory_events().ok())
        .and_then(|e| e.get("oom_kill").copied())
        .unwrap_or(0);

    // Owned copies so the exec task can be spawned with 'static data.
    let mut args = vec!["exec".to_string(), "-i".to_string(), container.to_string()];
    args.extend(run_cmd.iter().cloned());
    let input = test.input.clone();
    let mut exec_task = tokio::spawn(exec_case(args, input));

    // Baseline per case: the first sample only records the counter, later
    // samples compare against it so leftover events from a previous case can
    // never trigger a spurious promotion.
    let mut last_high: Option<u64> = None;
    let mut case_peak: u64 = 0;
    let mut poll = tokio::time::interval(MONITOR_POLL); // first tick fires immediately
    let deadline = tokio::time::sleep(CASE_TIMEOUT);
    tokio::pin!(deadline);

    let output = loop {
        tokio::select! {
            // Wall-clock guard. Without this an infinite loop holds its queue
            // semaphore permit forever and starves every other submission.
            _ = &mut deadline => {
                // Kill the submission process inside the container. We cannot
                // signal the `docker exec` child directly (it is not in the
                // container's PID namespace), so target the known run commands.
                let _ = Command::new("docker")
                    .args(["exec", container, "sh", "-c",
                        "pkill -9 -f '/app/run' 2>/dev/null; pkill -9 -f 'java' 2>/dev/null; pkill -9 -f 'main.py' 2>/dev/null; exit 0"])
                    .output()
                    .await;
                // Drop the exec task so tokio reaps the `docker exec` child.
                exec_task.abort();
                let _ = exec_task.await;
                return Ok(CaseResult {
                    verdict: "TLE".to_string(),
                    cpu_time_ms: case_start.elapsed().as_millis() as u64,
                    peak_memory_bytes: case_peak,
                    allocated_memory_bytes: allocated_for(*promoted, *tier),
                });
            }
            _ = poll.tick() => {
                if let Some(cg) = cg {
                    // Reactive trigger: did the kernel cross the soft watermark or memory exceed ~179.2 MiB (70%)?
                    if watch && !*promoted {
                        let cur = cg.memory_current().unwrap_or(0);
                        let events = cg.memory_events().ok();
                        let high = events.as_ref().and_then(|e| e.get("high").copied()).unwrap_or(0);
                        let high_crossed = last_high.map_or(false, |prev| high > prev);
                        last_high = Some(high);

                        let crossed = high_crossed || cur >= low_mem_high_watermark();
                        if crossed {
                            let signal =
                                MonitorSignal::new(cur, low_mem_high_watermark(), true);
                            if policy.should_promote(&signal) {
                                // The cgroup write IS the promotion: it lifts
                                // memory.high and memory.max to `max`.
                                //
                                // Do NOT follow this with `docker update
                                // --memory 0`: Docker treats 0 as "no change"
                                // and re-reconciles the cgroup back to the
                                // container's configured limit, restoring the
                                // 256 MiB cap one line after it was lifted.
                                // The container then dies at exactly the old
                                // ceiling while still reporting promoted=true.
                                match cg.promote_to_unlimited() {
                                    Ok(()) => {
                                        *promoted = true;
                                        *promotion_time_ms =
                                            started.elapsed().as_millis() as u64;
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[moderator] promotion failed for \
                                             {container}: {e}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                    // Per-case peak estimate (cheap read every tick).
                    if let Ok(cur) = cg.memory_current() {
                        if cur > case_peak {
                            case_peak = cur;
                        }
                    }
                } else if watch && !*promoted {
                    // Fallback when host cgroup directory is not directly reachable (e.g. Docker Desktop VM)
                    if let Ok(out) = Command::new("docker")
                        .args(["exec", container, "cat", "/sys/fs/cgroup/memory.current"])
                        .output()
                        .await
                    {
                        if out.status.success() {
                            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                            if let Ok(cur) = s.parse::<u64>() {
                                if cur > case_peak {
                                    case_peak = cur;
                                }
                                if cur >= low_mem_high_watermark() {
                                    let signal =
                                        MonitorSignal::new(cur, low_mem_high_watermark(), true);
                                    if policy.should_promote(&signal) {
                                        // No host cgroup path (Docker-Desktop-style
                                        // setups). `--memory 0` is a no-op here as
                                        // well: Docker reads 0 as "no change", so
                                        // only an explicit finite ceiling actually
                                        // raises the limit.
                                        let raised = Command::new("docker")
                                            .args([
                                                "update", container,
                                                "--memory", PROMOTED_MEMORY_CEILING,
                                                "--memory-swap", PROMOTED_MEMORY_CEILING,
                                                "--cpus", "0",
                                            ])
                                            .output()
                                            .await;
                                        match raised {
                                            Ok(o) if o.status.success() => {
                                                *promoted = true;
                                                *promotion_time_ms =
                                                    started.elapsed().as_millis() as u64;
                                            }
                                            Ok(o) => eprintln!(
                                                "[moderator] docker update failed for \
                                                 {container}: {}",
                                                String::from_utf8_lossy(&o.stderr).trim()
                                            ),
                                            Err(e) => eprintln!(
                                                "[moderator] docker update error for \
                                                 {container}: {e}"
                                            ),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            res = &mut exec_task => {
                break res??;
            }
        }
    };

    // If sampling during the loop did not capture a peak (e.g. fast task exit),
    // sample the current host cgroup, or query the container's internal cgroup file.
    if case_peak == 0 {
        if let Some(cg) = cg {
            if let Ok(cur) = cg.memory_current() {
                case_peak = cur;
            }
        }
    }
    if case_peak == 0 {
        if let Ok(out) = Command::new("docker")
            .args([
                "exec",
                container,
                "sh",
                "-c",
                "cat /sys/fs/cgroup/memory.peak 2>/dev/null || cat /sys/fs/cgroup/memory.current 2>/dev/null",
            ])
            .output()
            .await
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if let Ok(bytes) = s.parse::<u64>() {
                    case_peak = bytes;
                }
            }
        }
    }

    let wall_ms = case_start.elapsed().as_millis() as u64;
    let cpu_ms = match (cpu_start_usec, cg.and_then(|c| c.cpu_usage_usec().ok())) {
        (Some(before), Some(after)) => {
            let delta = after.saturating_sub(before);
            if delta == 0 {
                0
            } else {
                std::cmp::max(1, ((delta as f64) / 1000.0).round() as u64)
            }
        }
        _ => wall_ms,
    };
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let expected = test.expected.trim().trim_end_matches('\r').to_string();
    // The kernel's `oom_kill` counter is the authoritative signal that this
    // case actually hit `memory.max` and was killed. A non-zero *increase* over
    // the baseline means the submission genuinely exceeded the ceiling it was
    // given: either a non-promoting policy (Baseline/Predictive) ran it
    // bounded, or a Reactive/Hybrid promotion missed the spike.
    //
    // Without this check the kernel kill looks like any other non-zero exit and
    // is graded `RE`, which is indistinguishable from a segfault.
    let oom_killed = cg
        .and_then(|c| c.memory_events().ok())
        .and_then(|e| e.get("oom_kill").copied())
        .map_or(false, |n| n > oom_kill_start);
    let verdict = if oom_killed {
        "MLE"
    } else if !output.status.success() {
        "RE"
    } else if stdout == expected {
        "AC"
    } else {
        "WA"
    };
    let allocated = allocated_for(*promoted, *tier);
    Ok(CaseResult {
        verdict: verdict.to_string(),
        cpu_time_ms: cpu_ms,
        peak_memory_bytes: case_peak,
        allocated_memory_bytes: allocated,
    })
}

/// Run every test case of a submission in its container, watching and, when the
/// policy is reactive/hybrid, promoting the container live on memory pressure.
pub async fn run_submission(
    submission: &Submission,
    tier: &Tier,
    policy: &(dyn TierPolicy + Send + Sync),
    started: Instant,
) -> io::Result<RunOutcome> {
    let host_dir = write_source(submission)?;
    let result = submission_inner(submission, tier, policy, started, &host_dir).await;
    // host_dir is a temp dir; always clean it up, success or not.
    let _ = std::fs::remove_dir_all(&host_dir);
    result
}

async fn submission_inner(
    submission: &Submission,
    tier: &Tier,
    policy: &(dyn TierPolicy + Send + Sync),
    started: Instant,
    host_dir: &std::path::Path,
) -> io::Result<RunOutcome> {
    let (container, run_cmd) =
        start_and_compile(&submission.language, &submission.id, host_dir, tier).await?;

    // Locate the container's cgroup directory once for the whole submission.
    let can_promote = policy.can_promote();
    let cg = match locate_cgroup_dir(&container) {
        Ok(dir) => {
            let cg = CGroup::new(dir);
            // Arm the soft watermark so a Low-tier start can emit pressure
            // events (only meaningful when this policy can promote).
            if can_promote && *tier == Tier::Low {
                if let Err(e) = cg.set_memory_high(low_mem_high_watermark()) {
                    eprintln!(
                        "[moderator] failed to arm memory.high for {container}: {e}"
                    );
                }
            }
            Some(cg)
        }
        Err(e) => {
            eprintln!(
                "[moderator] cgroup not reachable for {container} ({e}); \
                 live promotion + peak metrics disabled for this submission"
            );
            None
        }
    };

    // Watch promotion whenever a Low start + a reactive-style policy hold.
    let watch = can_promote && *tier == Tier::Low;

    let mut results = Vec::new();
    let mut promoted = false;
    let mut promotion_time_ms = 0u64;

    for test in &submission.test_cases {
        let case = run_case_monitored(
            &container,
            &run_cmd,
            test,
            cg.as_ref(),
            watch,
            policy,
            tier,
            started,
            &mut promoted,
            &mut promotion_time_ms,
        )
        .await;
        match case {
            Ok(c) => results.push(c),
            Err(e) => {
                let _ = Command::new("docker")
                    .args(["rm", "-f", &container])
                    .output()
                    .await;
                return Err(e);
            }
        }
    }

    let _ = Command::new("docker")
        .args(["rm", "-f", &container])
        .output()
        .await;
    Ok(RunOutcome {
        results,
        tier_promoted: promoted,
        promotion_time_ms,
    })
}

async fn start_and_compile(
    language: &str,
    submission_id: &String,
    host_dir: &std::path::Path,
    tier: &Tier,
) -> std::io::Result<(String, Vec<String>)> {
    let image = image_for(language);
    let cname = format!("oj_{}", sanitize_id(submission_id));

    let _ = Command::new("docker")
        .args(["rm", "-f", &cname])
        .output()
        .await;

    let run_cmd = match language {
        "python" => vec!["python3".to_string(), "/app/main.py".to_string()],
        "java" => vec![
            "java".to_string(),
            // Heap ceiling is derived from the tier this container actually
            // gets. A fixed 512m inside a 256 MiB container told the JVM it
            // could use twice what the cgroup permits, so an allocation-heavy
            // submission was OOM-killed at the cap - and killed fast enough
            // that Reactive/Hybrid recorded a promotion they never got to use.
            // Sizing it from the tier removes that contradiction: a Low-tier
            // JVM now self-limits inside its budget instead of racing the
            // kernel for it.
            format!("-Xmx{}m", java_heap_mb(tier)),
            // Serial GC keeps the JVM's non-heap footprint small: one GC
            // thread instead of a G1 pool, no region metadata. The budget
            // outside the heap is what the 75% split in java_heap_mb() reserves.
            "-XX:+UseSerialGC".to_string(),
            "-cp".to_string(),
            "/app".to_string(),
            "Main".to_string(),
        ],
        "c" | "cpp" | "c++" => vec!["/app/run".to_string()],
        _ => vec![],
    };
    let tier_limits = get_tier_limits(tier);
    let mut start_args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        cname.to_string(),
        "--network=none".to_string(),
    ];
    start_args.extend(tier_limits);
    start_args.extend([
        image.to_string(),
        "sh".to_string(),
        "-c".to_string(),
        "sleep infinity".to_string(),
    ]);
    let start = Command::new("docker").args(start_args).output().await?;
    if !start.status.success() {
        let err = String::from_utf8_lossy(&start.stderr);
        eprintln!("[docker run error for {cname}]: {err}");
        return Err(std::io::Error::other(format!(
            "Error in starting container {cname}: {err}"
        )));
    }
    let source_file = source_filename(language);
    let cp = Command::new("docker")
        .args([
            "cp",
            &format!("{}/{}", host_dir.display(), source_file),
            &format!("{}:/app/{}", cname, source_file),
        ])
        .output()
        .await?;
    if !cp.status.success() {
        let _ = Command::new("docker")
            .args(["rm", "-f", &cname])
            .output()
            .await?;
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&cp.stderr).to_string(),
        ));
    }

    let compile_cmd: &[&str] = match language {
        "java" => &["sh", "-c", "javac /app/Main.java -d /app"],
        "c" => &["sh", "-c", "gcc -o /app/run /app/main.c"],
        "cpp" | "c++" => &["sh", "-c", "g++ -o /app/run /app/main.cpp"],
        _ => &[],
    };
    if !compile_cmd.is_empty() {
        let comp = Command::new("docker")
            .args(["exec", &cname])
            .args(compile_cmd)
            .output()
            .await?;
        if !comp.status.success() {
            let _ = Command::new("docker")
                .args(["rm", "-f", &cname])
                .output()
                .await?;
            return Err(std::io::Error::other(
                String::from_utf8_lossy(&comp.stderr).to_string(),
            ));
        }
    }

    Ok((cname.clone(), run_cmd))
}

#[cfg(test)]
mod sanitize_tests {
    use super::sanitize_id;

    /// Docker requires `[a-zA-Z0-9][a-zA-Z0-9_.-]*`. Anything else makes the
    /// container name invalid and the submission fails with an opaque `SE`.
    fn assert_valid(id: &str) -> String {
        let s = sanitize_id(id);
        let mut chars = s.chars();
        let first = chars.next().expect("sanitized id is never empty");
        assert!(
            first.is_ascii_alphanumeric(),
            "id {id:?} -> {s:?} must start with an alphanumeric character"
        );
        for c in s.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-',
                "id {id:?} -> {s:?} contains illegal character {c:?}"
            );
        }
        s
    }

    #[test]
    fn plain_ids_pass_through_unchanged() {
        for id in ["sub-1", "sub_42", "bench_P4_trips_bfs_cpp_baseline", "a.b-c_d"] {
            assert_eq!(assert_valid(id), id, "already-valid id must be preserved");
        }
    }

    /// Rewritten ids get a `<name>_<hash8>` suffix; only the stem is stable.
    fn stem(id: &str) -> String {
        let s = assert_valid(id);
        match s.rsplit_once('_') {
            Some((head, tail)) if tail.len() == 8 && tail.chars().all(|c| c.is_ascii_hexdigit()) => {
                head.to_string()
            }
            _ => s,
        }
    }

    #[test]
    fn plus_sign_from_language_names_is_neutralised() {
        // `C++` is a real language label and was the original trigger: it
        // produced the invalid container name `oj_cvC++` and an opaque `SE`.
        for id in ["oj_cvC++", "C++", "cpp_C++_v2"] {
            let s = assert_valid(id);
            assert!(!s.contains('+'), "id {id:?} -> {s:?} still contains '+'");
            assert!(!s.starts_with(['_', '.', '-']), "{s:?} starts with a separator");
        }
        // The exact stem is cosmetic; the contract is validity and stability.
        assert_eq!(assert_valid("oj_cvC++"), assert_valid("oj_cvC++"));
    }

    #[test]
    fn path_separators_and_spaces_cannot_escape_the_temp_dir() {
        assert_eq!(stem("sl/ash"), "sl_ash");
        assert_eq!(stem("sp ace"), "sp_ace");
        // The traversal components are destroyed, so the result cannot escape
        // the temp dir even before the prefix is applied.
        assert_eq!(stem("../../etc/passwd"), "etc_passwd");
        assert!(!sanitize_id("../../etc/passwd").contains(".."));
    }

    #[test]
    fn leading_separators_are_stripped() {
        assert_eq!(stem(".hidden"), "hidden");
        assert_eq!(stem("--flag"), "flag");
    }

    #[test]
    fn runs_of_illegal_bytes_collapse() {
        assert_eq!(stem("a   b"), "a_b");
        assert_eq!(stem("a++b"), "a_b");
    }

    #[test]
    fn empty_and_all_illegal_ids_still_produce_a_valid_name() {
        assert_eq!(assert_valid(""), "submission");
        assert_eq!(assert_valid("///"), "submission");
        assert_eq!(assert_valid("..."), "submission");
    }

    #[test]
    fn distinct_ids_do_not_collide() {
        // Collapsing must not merge ids that differ only in illegal bytes:
        // a collision would let one submission's `docker rm -f` tear down
        // another submission's running container.
        assert_ne!(assert_valid("x/y"), assert_valid("x_y"));
        assert_ne!(assert_valid("a+b"), assert_valid("a b"));
        // ...and the suffix must be stable, not random per call.
        assert_eq!(assert_valid("x/y"), assert_valid("x/y"));
    }
}

#[cfg(test)]
mod java_heap_tests {
    use super::{java_heap_mb, low_mem_hard_limit, JAVA_HEAP_PCT_OF_LOW_TIER};
    use crate::policy::Tier;

    /// The whole point of sizing the heap from the tier: a Low-tier JVM must be
    /// told it can use *less* than its container, never more. The old constant
    /// (512m) was twice the 256 MiB container and could not be honoured, so an
    /// allocation-heavy submission was OOM-killed rather than running.
    #[test]
    fn low_tier_heap_stays_inside_the_container() {
        let low_mb = low_mem_hard_limit() / (1024 * 1024);
        let heap = java_heap_mb(&Tier::Low);
        assert!(
            heap < low_mb,
            "Low-tier heap {heap}m must be strictly below the {low_mb}m container"
        );
        assert_eq!(heap, low_mb * JAVA_HEAP_PCT_OF_LOW_TIER / 100);
    }

    /// A JVM needs memory outside the heap (metaspace, stacks, code cache), so
    /// the reserve has to be big enough to actually run in.
    #[test]
    fn low_tier_reserves_room_outside_the_heap() {
        let low_mb = low_mem_hard_limit() / (1024 * 1024);
        let reserve = low_mb - java_heap_mb(&Tier::Low);
        assert!(
            reserve >= 32,
            "only {reserve}m left for non-heap JVM memory at a {low_mb}m tier"
        );
    }

    /// High is unbounded, so it keeps the historical ceiling instead of none at
    /// all - an unbounded heap lets one runaway submission take the whole host.
    #[test]
    fn high_tier_keeps_a_ceiling_and_is_larger_than_low() {
        let low_mb = low_mem_hard_limit() / (1024 * 1024);
        assert_eq!(java_heap_mb(&Tier::High), low_mb * 2);
        assert!(java_heap_mb(&Tier::High) > java_heap_mb(&Tier::Low));
    }
}
