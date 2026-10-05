# RAAS-OCJS — Experimental Programme and Paper Plan

**Status:** authoritative plan. The previous harness, its result CSVs, and every paper draft have been
removed from the repository. Nothing in git history should be cited. Everything the paper will assert
must be re-derived under this document.

**How to read this.** Part I defines the paper we are going to write and every element it will contain.
Part II states how the system works. Part III is the blocking work. Part IV is the experiments — each
specifies its procedure, its data schema, the paper element it produces, and its kill condition.
Parts V–VIII are hygiene, threats, sequencing and decisions.

**The organising rule:** no experiment exists for its own sake. Every experiment below produces a named
table or figure in the paper, or it does not run.

---

# Part I — The paper

## I.1 Format and constraints

- **Class:** `Conference_paper/IEEEtran.cls`, `\documentclass[conference]{IEEEtran}`.
- **Template:** `Conference_paper/RAAS_OCJS.tex` (blank; to be written from scratch).
- **Length:** IEEE conference, typically 6–8 pages including references, two-column.
- **Title and abstract may not contain math or special symbols.**
- All template guidance text must be removed before submission.

## I.2 The claim set

The paper asserts four things and nothing more. Each is separately falsifiable; the paper must survive
any one being weakened, and must say so if one falls.

| ID | Claim | Evidence element | Kill condition |
|---|---|---|---|
| **C1** | Contest submission demand is bimodal — a low median with a thin heavy tail | T2, F2 | demand is broad; then no tiering helps |
| **C2** | Light-start + promotion is **safe** — it never causes a memory-caused failure that worst-case provisioning would have prevented | T5 | any such failure |
| **C3** | Adaptive sustains **strictly more concurrent graded work at the same RAM and the same correctness** than any static configuration | T3, T4, F4 | best static cell matches adaptive |
| **C4** | Prediction **alone** cannot match reactive, because peak demand is not determined by source | T6 | the oracle arm matches reactive |

**The framing to avoid.** Do not present "% of bytes saved" as the headline. Bytes are a proxy and a
reviewer will discount them. The headline is **throughput at fixed RAM and fixed correctness**.
Savings are the *mechanism*; capacity is the *result*.

**The comparison to make.** Not one worst-case strawman, but a **sweep of real configurations**
(fixed limit × fixed pool size), which is what DOMjudge, DMOJ and Judge0 actually deploy.

## I.3 Every table the paper will contain

Columns are fixed here so the harness can be written to emit exactly these, with no post-hoc assembly.

**T1 — Corpus composition.** *Source: E1.*

| Column | Meaning |
|---|---|
| source | CodeContests / synthetic / other |
| problems | distinct problem count |
| submissions | total submissions |
| cpp / python / java / c | per-language counts |
| p50_rss_mib, p99_rss_mib, max_rss_mib | peak RSS summary |
| above_watermark | count exceeding the tier watermark |

**T2 — Demand distribution.** *Source: E1.* The C1 evidence.

| Column | Meaning |
|---|---|
| statistic | min / p25 / p50 / p75 / p90 / p95 / p99 / max |
| rss_mib | peak RSS at that percentile |
| pct_below_25 / pct_below_256 / pct_below_1024 / pct_below_2048 | share under each ceiling |

**T3 — Static configuration sweep.** *Source: E2.* The frontier C3 is measured against.

| Column | Meaning |
|---|---|
| limit_mib | fixed per-submission ceiling ∈ {128,256,512,1024,2048} |
| pool_size | fixed admission ∈ {4,7,14,16,32,56} |
| reservation_mib | `limit × pool` actually committed |
| throughput_per_s | graded submissions per second |
| p50_ms, p95_ms, p99_ms | turnaround |
| mem_failures | memory-caused failures per 1000 |
| feasible | `mem_failures == 0` |

**T4 — Policy comparison at equal correctness.** *Source: E3.* The C3 headline.

| Column | Meaning |
|---|---|
| policy | baseline / predictive / hybrid / reactive |
| admission | mean and peak admitted concurrency |
| reservation_mib | mean and peak reservation held |
| throughput_per_s | at zero memory-caused failures |
| p95_ms | turnaround |
| promotions | count and rate |
| mem_failures | must be 0 for every row, or the row is invalid |

**T5 — Correctness envelope of promotion.** *Source: E4, E6.* The C2 evidence.

| Column | Meaning |
|---|---|
| case_class | at-watermark / over-watermark / fast-alloc / late-spike / fragmented |
| language | python / cpp / java / c |
| verdict_reactive | verdict under light-start + promotion |
| verdict_baseline | verdict under 2048 MiB (the oracle) |
| agree | boolean |
| promotion_latency_ms | acceptance → `max` written |
| peak_rss_mib | measured peak |
| oom_kill_delta | kernel counter delta for this submission |
| race_lost | promoted but still died |

**T6 — Prediction, shipped vs oracle.** *Source: E5.* The C4 evidence.

| Column | Meaning |
|---|---|
| arm | shipped / oracle / reactive |
| false_heavy_rate | flagged heavy, measured light (wasted reservation) |
| false_light_rate | flagged light, measured heavy (correctness risk) |
| reservation_mib | reservation held |
| reservation_floor_mib | the floor prediction cannot go below |
| mem_failures | with reactive recovery disabled |

**T7 — Tier floor.** *Source: E7.*

| Column | Meaning |
|---|---|
| language | python / cpp / java / c |
| tier_mib | 64 / 128 / 256 |
| verdict | pass / fail |
| failure_kind | cgroup-oom / runtime-abort / none |
| signature | exit code and stderr marker |

**T8 — Cloud provisioning.** *Source: E8 if executed, otherwise omitted.* **Measured only.** A row
that is arithmetic rather than measured must be marked as such in the caption or removed.

| Column | Meaning |
|---|---|
| instance | e2-standard-4, asia-south1 |
| hourly_usd | verified against the Cloud Billing API, with date |
| usable_mib | usable host budget |
| pods_baseline / pods_adaptive | concurrent slots at each tier |
| vms_burst | fleet size for the stated burst |
| usd_per_hour | cluster cost |
| measured_or_derived | explicit marker per row |

**T9 — Headline summary.** *Source: all.* Four rows, one per claim, each with the single number that
supports it and a pointer to its table. This is the only table allowed to mix sources, and every cell
must cite where it came from.

## I.4 Every figure the paper will contain

**F1 — System architecture.** Tiers, the cgroup files, the watermark, the promotion path, the four
policies. Static diagram. Must show that promotion is **memory-only**, and distinguish a Tier-2
*placement* from a *promotion*.

**F2 — Demand distribution.** Log-scale histogram plus CDF of peak RSS. This is the visual proof of C1
and the single most important figure in the paper. Plotted from E1 raw data, not from summary
statistics.

**F3 — Reservation over time.** Two traces under identical load: static worst-case and adaptive.
Shows reservation held against actual use. This is where "waste" becomes visible and is what justifies
the mechanism.

**F4 — The tension frontier.** Throughput (y) against memory-caused failures (x), or throughput against
reservation, with every static cell from E2 plotted and adaptive marked. This is the C3 result. If
adaptive does not sit on the frontier, the paper must say so.

**F5 — Promotion latency.** Distribution of acceptance→promoted latency, plus the allocation-rate race
result from E4 (latency and success/failure as a function of allocation MiB/s). This is the C2
quantitative bound.

**F6 — Burst response.** Queue wait and turnaround CDFs under a fixed burst schedule, **measured
end-to-end**. If the burst cannot be driven end-to-end, the figure is omitted and the claim is not made.

## I.5 Traceability

Every claim → experiment → element → section. Nothing may appear in the paper without a row here.

| Claim | Experiments | Tables | Figures | Paper section |
|---|---|---|---|---|
| C1 | E1 | T1, T2 | F2 | Motivation / workload characterisation |
| C2 | E4, E6 | T5 | F5 | Mechanism and safety |
| C3 | E2, E3 | T3, T4, T9 | F3, F4, F6 | Results |
| C4 | E5 | T6 | — | Prediction analysis |
| — | E7 | T7 | — | Boundary / limits |
| — | E8 | T8 | — | Deployment |

---

# Part II — How the system works

## II.1 Tiers

| | Memory | CPU | Where |
|---|---|---|---|
| **Tier 1 (low)** | 256 MiB hard; `memory.high` armed at 70% = **179.2 MiB** | 1.0 core | start point for Reactive/Hybrid |
| **Tier 2 (high)** | 2048 MiB | 2.0 cores | static worst case; also a direct placement target |
| **Promotion** | `memory.high` + `memory.max` → `max` (8 GiB ceiling const) | **untouched** | mid-execution, no restart |

Promotion is **memory-only**; `set_cpu_max` is dead code reserved for a follow-up. A Tier-2
*placement* is a different ceiling from a *promotion*. Do not conflate them in text or diagrams.

## II.2 Policies (`server/src/policy.rs`)

| Policy | Initial tier | Promotes | Failure mode |
|---|---|---|---|
| `baseline` | always High | no | none, but worst-case reservation |
| `predictive` | classifier | **no** | **hard MLE on misroute — no recovery** |
| `reactive` | always Low | yes | promotion latency; race on fast allocation |
| `hybrid` | classifier | yes | as predictive, with recovery |

## II.3 Promotion loop (`server/src/docker.rs`)

Polls every **2 ms**; arms `memory.high` at 179.2 MiB; triggers on the `memory.events` `high` counter
crossing **or** `memory.current ≥ 179.2 MiB`; writes `max` to `memory.high`/`memory.max`. Per-case
timeout 10 s. Java heap pinned to 75% of the Low tier.

## II.4 Admission control — the blocking gap

`server/src/queue.rs`:

```rust
const MAX_CONCURRENT: usize = 16;   // plain Semaphore, tier-independent
```

There is **no memory-aware admission** anywhere in the server. The tier changes what a *container* may
allocate; it does not change *how many run*.

Consequence: at 16 × 256 MiB = 4 GiB on a 15 GiB host, the host sits at ~28% RAM and is **never under
memory pressure at any load**. Arrivals queue on the semaphore, not on memory. The scenario the thesis
is about cannot currently occur. This is why **E3 requires a code change**, and why every queueing
number produced so far was simulated.

## II.5 Evidence grades

Every number in the paper must be labelled with one of:

- **M** — measured end-to-end on a running system.
- **D** — derived by arithmetic from measured inputs (the inputs must be stated).
- **S** — simulated (a discrete-event model). **Not permitted for any headline claim.**

---

# Part III — Blocking work

## E0 — Instrument, configure, rebuild the harness

Nothing downstream is trustworthy until this is done. No result may be claimed from E0 itself.

**E0.1 Per-submission record.** Emit one structured record per graded submission (JSON Lines):

```
submission_id, problem_id, language, policy,
tier_start, tier_end, promoted, promotion_count,
t_accept, t_container_start, t_first_case, t_last_case, t_done,
promotion_latency_ms,            // acceptance -> memory.max written
queue_wait_ms,                   // enqueue -> dispatch
peak_rss_mib,                    // max memory.current observed
peak_allocated_mib,              // configured ceiling at peak
verdict, exit_code, stderr_marker,
cgroup_memory_events_before,     // low/high/max/oom/oom_kill
cgroup_memory_events_after,
host_ram_available_mib, host_load1
```

**E0.2 Kernel counters per submission, not per run.** `memory.events` and `memory.max` are ground
truth. The judge's own `tier_promoted` flag is a self-report and has already been wrong once.

**E0.3 Configurable admission.** Replace the `MAX_CONCURRENT` constant with a runtime policy:
`fixed(N)` for the static sweep, and `memory_derived(reserve_mib)` which admits while
`(available_ram - reserve) >= tier_reservation`. This is what makes E3 possible.

**E0.4 Configurable tiers.** Tier sizes and the watermark percentage via env, so E7 can sweep them
without a rebuild.

**E0.5 Rebuild the harness.** A new script that can: build and validate a corpus; drive `/submit` at a
controlled arrival process (fixed rate or burst schedule); run a policy × limit × pool matrix
unattended; and write the JSONL above plus the aggregate CSVs for T1–T9. **No simulation subcommand.**
If a number cannot be measured, it does not go in the paper.

**E0.6 Provenance.** Every result directory gets a `PROVENANCE.md`: corpus name and hash, submission
count, host, instance, commit, config values, date, operator. A result set without one is not citable.

**E0.7 Delete the AWS constants.** Instance identity and price become parameters defaulting to
`e2-standard-4` / `asia-south1` / `$0.160969` (re-verify against the Cloud Billing API at run time and
record the date). The generator must never emit a provider it was not told to.

**Gate:** E1–E8 may not start until a single submission's record can be traced from HTTP receipt to
kernel counters without manual interpretation.

---

# Part IV — Experiments

Each is: question → hypothesis → procedure → corpus and controls → data → paper element → kill
condition.

## E1 — Workload characterisation → **C1**, T1, T2, F2

**Question.** Is peak memory demand bimodal with a thin heavy tail?

**Hypothesis.** `p50 ≪ p99 ≪ ceiling`, with a large majority under 25 MiB.

**Procedure.**
1. Build a corpus of **≥1000 submissions across ≥100 problems and ≥4 languages**.
2. Run every submission at **2048 MiB** so the tier cannot truncate the measurement.
3. Record `peak_rss_mib` per submission (E0.1 schema).
4. Compute T1 and T2; plot F2 from the raw values.

**Corpus.** CodeContests is the primary source. Synthetic heavy submissions may be included **only if
T1 marks them as a separate source row** and F2 distinguishes them. Real heavy submissions from a live
contest are the ideal addition; if unavailable, say so.

**Controls.** Re-run a sample of light submissions at 256 MiB to confirm the tier does not change the
measured peak for submissions that fit.

**Paper element.** T1, T2, F2.

**Kill condition.** If `p50` is not small relative to 2048 MiB, or >10% of submissions exceed 512 MiB,
C1 is false. **Report it either way** — this is the foundation and it must be honest.

**Effort.** Small. Do this first.

## E2 — Static sweep → **C3**, T3, F4

**Question.** Is there any single static configuration that is both safe and efficient?

**Hypothesis.** There is a tension frontier, not an optimum.

**Procedure.**
1. For each `limit ∈ {128,256,512,1024,2048}` MiB and `pool ∈ {4,7,14,16,32,56}`, run the corpus at a
   fixed arrival schedule.
2. Record throughput, P50/P95/P99 turnaround, memory-caused failures, and committed reservation.
3. Mark each cell feasible iff `mem_failures == 0`.
4. Plot F4 from the feasible cells.

**Controls.** Identical corpus, host and arrival schedule in every cell. Fixed seed. Record host state.

**Paper element.** T3, F4.

**Kill condition.** If some static `limit` achieves both zero memory-caused failures **and** adaptive's
throughput, C3 is false and the paper must say so. Run this early precisely because it can end the
project.

**Effort.** Large — up to 30 cells, each a full corpus run. This is the main compute cost.

## E3 — Adaptive vs static under memory-driven admission → **C3**, T4, F3, F6, T9

**Question.** At fixed RAM and equal correctness, does adaptive sustain more concurrent work?

**Hypothesis.** Yes — strictly more, because reservation converts directly into admitted concurrency.

**Procedure.**
1. Enable `memory_derived` admission (E0.3).
2. Replay the E2 corpus and arrival schedule under each of the four policies.
3. Measure admitted concurrency, reservation over time, throughput, P95, failures, promotions.
4. Compare against the E2 frontier at equal correctness.

**Controls.** Same corpus, schedule and host as E2. **Also run `baseline` under the same dynamic
admission**, so the only variable is the tier policy and not the admission policy. Every T4 row must
have `mem_failures == 0`; a row with failures is invalid and is reported as such.

**Metrics.** The headline is **submissions/hour at the same RAM and the same correctness**. Secondary:
peak admitted concurrency, mean and peak reservation, promotion rate.

**Paper element.** T4, F3, F6, and the T9 headline row for C3.

**Kill condition.** If adaptive's sustainable rate is not materially above the best feasible static
cell, C3 fails. If raising admission with 256 MiB tiers produces *any* memory-caused failure, C2 fails.

**Effort.** Large. Blocked on E0.3.

## E4 — Correctness envelope of promotion → **C2**, T5, F5

**Question.** Can light-start + promotion ever cause a memory-caused failure that 2048 MiB would have
prevented?

**Hypothesis.** No, within a boundable allocation rate.

**Procedure.** Build an adversarial suite; each case runs under both `reactive` and `baseline`:

| Case class | Description |
|---|---|
| at-watermark | steady allocation landing exactly on 179.2 MiB |
| over-watermark | steady allocation past it (400, 900 MiB) |
| **fast-alloc** | tight loop allocating in large blocks, **faster than the 2 ms poll** |
| late-spike | quiet for N seconds, then a large allocation |
| fragmented | many small allocations summing past the ceiling |
| runtime-floor | a trivial program at 64/128/256 MiB per language |

For fast-alloc, sweep the allocation rate to find the rate at which promotion first loses.

**Controls.** `baseline` at 2048 MiB is the **correctness oracle**: a case that fails under both is not
a tiering failure and must not be counted as one.

**Metrics.** Verdict agreement with the oracle; `oom_kill` delta per submission; promotion latency
distribution; the allocation rate (MiB/s) at which the race is first lost.

**Paper element.** T5, F5.

**Kill condition.** Any case that fails under `reactive`, passes under `baseline`, has a memory-caused
verdict and a non-zero `oom_kill` delta → C2 is false. Report the rate threshold either way; if
promotion only fails above a high rate, that is a bounded, statable limitation.

**Effort.** Medium, but delicate — repeated trials are required because the monitor is asynchronous.

## E5 — Is prediction inherently insufficient? → **C4**, T6

**Question.** Is predictive-only worse because of *this model*, or because of *prediction*?

**Hypothesis.** Even with perfect prediction, predictive-only cannot match reactive, because it must
reserve worst-case for everything it flags.

**Procedure.** Three arms on the same corpus:
1. `predictive` with the shipped classifier;
2. `predictive` with an **oracle label** (the true peak from E1) — the best possible predictor;
3. `reactive`.

For arms 1–2, disable reactive recovery so the prediction is the only defence.

**Controls.** The oracle arm is the control. Any gap between arm 2 and arm 3 is a property of
prediction, not of the model.

**Metrics.** False-heavy rate (wasted reservation), false-light rate (correctness risk), reservation
held, and the reservation floor prediction cannot go below.

**Paper element.** T6.

**Kill condition.** If the oracle arm matches reactive, C4 is false and the honest conclusion is
"this classifier is under-trained", not "prediction is insufficient". Report either outcome.

**Effort.** Medium. Requires E1's true peaks as labels.

## E6 — Mechanism verification at the kernel → supports **C2**, T5

**Question.** Does the promotion trigger fire exactly at the configured watermark, and is the cgroup
write the promotion?

**Procedure.**
1. Reproduce the boundary cases: 170 MiB must **not** promote; 185 MiB **must**.
2. Sample `memory.max` / `memory.high` / `memory.events` before, during and after.
3. Assert `memory.max` goes `268435456` → `max` and that the judge's `tier_promoted` agrees.

**Controls.** **Both directions are required** — a non-promoting policy at the same demand must fail,
and a promoting policy must succeed. One direction alone proves nothing.

**Paper element.** T5 (mechanism rows); supports F5.

**Kill condition.** Promotion firing away from the watermark, or any `oom_kill` delta in a promoting
case, falsifies the mechanism.

**Effort.** Small.

## E7 — Tier floor → T7

**Question.** What is the smallest tier that is safe for every supported language?

**Procedure.** Sweep `{64,128,256}` MiB per language with a trivially correct program. Separate
**runtime** failure (JVM heap, interpreter stack) from **demand** failure (the program genuinely needed
more).

**Controls.** The same program at 2048 MiB, always correct.

**Metrics.** Failure kind per (language, tier): cgroup OOM vs in-process runtime abort, with exit code
and stderr marker.

**Paper element.** T7.

**Kill condition.** If no language fails at 128 MiB for a structural reason, the floor claim is wrong
and the achievable ratio is 16×, not 8×.

**Effort.** Small.

## E8 — Multi-host / cloud → T8 (only if the single-host result holds)

**Question.** Does the advantage survive when admission spans more than one host?

**Procedure.** Deploy to GCP `e2-standard-4` in `asia-south1`; run the E2/E3 matrix there; if feasible,
add a second host behind a dispatcher computing admission from aggregate reservation.

**Controls.** Same corpus and schedule as the single-host runs, so the only difference is the host.

**Metrics.** Sustained rate, correctness, reservation, coordination overhead.

**Paper element.** T8 — **measured rows only**. Any derived row must be marked `D` in the caption.

**Kill condition.** If the advantage vanishes with coordination, state the result as single-host only.

**Effort.** Large, and the only experiment with a recurring cost. Record the hourly and monthly figure
and the teardown command in the paper's deployment section.

---

# Part V — Measurement hygiene

Eight traps, all hit once already. Cheap to avoid, expensive to discover in review.

1. **Sample kernel counters per submission.** `memory.events` and `memory.max` are ground truth; the
   judge's flag is a self-report that has been wrong before.
2. **Never compare across corpora without saying so.** The 87.5%-vs-81.49% divergence was exactly
   this: a corpus peaking at 47.6 MiB cannot exercise a 179.2 MiB watermark, so its zero promotions
   were arithmetically forced, not measured.
3. **Label every number M / D / S.** A simulated queue result and a measured one must never share a
   column unlabelled.
4. **Keep admission control visible in every result.** It is the difference between a busy system and
   a contended one, and it dominated every previous result.
5. **Fixed seeds, fixed schedules, recorded host state.** CPU governor, turbo, background load,
   container image digests, commit hash.
6. **Report the tension, not a winner.** Where a setting is safer but slower, plot the frontier.
7. **Distinguish four failure kinds.** Wrong answer, timeout, cgroup OOM kill, in-process runtime
   abort. Conflating them is how a mechanism gets credited for something it did not do.
8. **A result set with no provenance note is not citable.** See E0.6.

**Repetition.** The promotion monitor is an asynchronous 2 ms poll. Every promotion measurement needs
repeated trials and a reported distribution, never a single sample.

---

# Part VI — Threats to validity

- **Single host, single instance type.** Cache, memory bandwidth and core count all move the promotion
  race.
- **Synthetic heavy submissions were authored to trigger the mechanism.** They cannot evidence that
  promotion triggers in the wild. If they appear, T1 must separate them and F2 must distinguish them.
- **Language coverage.** C is a small fraction of the corpus; any per-language conclusion about C is
  noise unless E1 deliberately balances it.
- **Classifier leakage.** Verify no test-case or expected-output signal reaches the model, and use
  grouped cross-validation by problem.
- **Time.** No experiment here addresses long-run stability (hours), which a real contest needs. State
  this as a limitation rather than implying it.
- **Arrival process.** A synthetic arrival schedule is not a real contest. State which schedule was
  used.

---

# Part VII — Sequencing and definition of done

| Order | Item | Why |
|---|---|---|
| 1 | **E0** instrument, configure, rebuild harness | nothing downstream is trustworthy without it |
| 2 | **E1** workload characterisation | cheapest test of the core premise; can kill the project in a day |
| 3 | **E6** mechanism verification | small, and it validates the instrument itself |
| 4 | **E4** correctness envelope | the only claim that can *end* the project |
| 5 | **E2** static sweep | defines the frontier E3 is measured against |
| 6 | **E3** adaptive vs static under dynamic admission | the central experiment |
| 7 | **E5** oracle-prediction comparison | settles C4 as a property of prediction |
| 8 | **E7** tier floor | bounds the supported range |
| 9 | **E8** cloud / multi-host | only if the single-host result holds |

**Stop conditions.**
- E1 shows demand is not bimodal → stop, re-scope the whole project.
- E2 finds one static configuration matching adaptive at equal correctness → stop, report that.
- E4 shows an unbounded promotion race → rework the mechanism before any throughput claim.

**Definition of done for the paper.** Every claim in I.2 has a table or figure in I.3/I.4; every number
carries an M/D/S label; every result directory has a `PROVENANCE.md`; the `.tex` compiles with zero
errors, zero undefined references and zero overfull boxes; and no number in the paper traces to
anything outside the repository's current tree.

---

# Part VIII — Decisions required

1. **Admission control is not memory-aware, and E3 needs it to be.** Changing `MAX_CONCURRENT` from a
   constant to a memory-derived bound is a code change to `server/src/queue.rs`. In scope?
2. **Synthetic heavy submissions.** May they appear in headline results, given they were authored to
   trigger the mechanism? Recommendation: permitted in T5 (mechanism) and F5; excluded from T1–T4 and
   F2/F4 unless T1 separates them as a distinct source.
3. **How hard to push "beats existing judges".** Doing it properly means modelling or citing real
   DOMjudge / DMOJ / Judge0 configurations in E2, rather than one worst-case baseline. In scope?
4. **Corpus scale.** E1 wants ≥1000 submissions across ≥100 problems; E2–E3 multiply that by up to 30
   cells. Is there a compute budget, or do we size the corpus to what one machine can run overnight?
5. **Authorship and venue.** The template has six author blocks. Who is on the paper, in what order,
   and which venue's page limit and deadline are we targeting? This fixes the length budget in I.1.