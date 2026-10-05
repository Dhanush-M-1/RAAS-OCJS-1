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
- **Length: 6 pages including references, two-column. This is now fixed** and it is the binding
  constraint on the whole design — see I.6 for the element budget it forces.
- **Title and abstract may not contain math or special symbols.**
- All template guidance text must be removed before submission.

## I.1b Authorship (confirmed from the current PDF)

Five authors in the IEEE block, in this order:

| # | Name | Email | Affiliation |
|---|---|---|---|
| 1 | Bharath Aashish R | bharathaashish@gmail.com | Dept. of Computer Science & Engineering, Easwari Engineering College, Chennai |
| 2 | Dhanush M | dhanush.m.0808@gmail.com | Dept. of Computer Science & Engineering, Easwari Engineering College, Chennai |
| 3 | Hemanthkumar K | hemanthkumar2k04@gmail.com | Dept. of Computer Science & Engineering, Easwari Engineering College, Chennai |
| 4 | Iniyaa P | iniyaapaari@gmail.com | Dept. of Computer Science & Engineering, Easwari Engineering College, Chennai |
| 5 | Indumathy P | indumathy.p@eec.srmrmp.edu.in | Assistant Professor, Dept. of CSE, Easwari Engineering College, Chennai |

The template provides six blocks, so one is unused. Two things to settle before submission: the
guide's affiliation line currently reads "Assistant Professor, Dept. of CSE" while the others read
"Dept. of Computer Science & Engineering" — normalise the wording; and confirm the guide's position
is intended (last is conventional, second is also common).

## I.1a Testbed

**The testbed is a Google Cloud Platform VM, and nothing else.** The deployment target is a single
`e2-standard-4` (4 vCPU, 16 GB) in `asia-south1`, provisioned from `deploy/terraform`, reached only
over an IAP tunnel. Every number in the paper must come from that instance.

No developer workstation, laptop or bare-metal host appears anywhere in the paper — not as a testbed,
not as a calibration source, not as a footnote. The paper reports one environment; stating two would
invite the question of which result came from which, and the corpus, cache and core-count differences
between them are large enough to move the promotion race.

Consequences for the experiments below:

- **Usable host budget is the VM's**, not a workstation's: derive it from the instance shape at run
  time and record it, rather than assuming a fixed figure.
- **E8 is a scaling experiment across identical VMs**, not a comparison between a local host and a
  cloud one.
- **Corpus construction is measured on the VM.** There is no separate calibration corpus from a
  different machine to fold in.
- Every result directory's `PROVENANCE.md` records the instance type, region and zone.

**Cost note.** The VM bills while it exists. Every experiment that provisions one must record the
hourly rate, the elapsed hours, the total, and the teardown command in the paper's deployment
section.

## I.2 The claim set

The paper asserts four things and nothing more. Each is separately falsifiable; the paper must survive
any one being weakened, and must say so if one falls.

| ID | Claim | Evidence element | Kill condition |
|---|---|---|---|
| **C1** | Contest submission demand is bimodal — a low median with a thin heavy tail | T2, F2 | demand is broad; then no tiering helps |
| **C2** | Light-start + promotion is **safe** — it never causes a memory-caused failure that worst-case provisioning would have prevented | T5 | any such failure |
| **C3** | Adaptive sustains **strictly more concurrent graded work at the same RAM and the same correctness** than any static configuration | T3, T4, F4 | best static cell matches adaptive |
| **C4** | Prediction **alone** cannot match reactive, because peak demand is not determined by source | T6 | the oracle arm matches reactive |
| **C5** | The routing model is reproducible and its errors are quantified — trained on measured-memory labels, evaluated problem-disjoint, and reported as a misroute rate decomposed into genuine over-limit failures and boundary artefacts | T10–T13, F7 | the model cannot be retrained to the reported numbers, or the error decomposition does not survive |

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

**T10 — Training corpus.** *Source: E9.* Required: the model is only interpretable next to its labels.
| Column | Meaning |
|---|---|
| source | dataset name and provenance |
| submissions | total rows |
| unique_problems | distinct problems |
| label_rule | the exact rule that produced Light/Heavy |
| label_basis | **measured** memory vs **heuristic** (length/difficulty) |
| light / heavy | class counts and ratio |
| ambiguous_band | rows dropped because the label was not decidable |
| parse_errors | rows filtered by the feature extractor, and the rate |
| on_disk | corpus size |

**T11 — Model performance.** *Source: E9.* One row per trained model.
| Column | Meaning |
|---|---|
| model | unified multi-language, or specialized C++/Java/Python/C |
| features | feature-vector length |
| cv_accuracy | grouped cross-validation accuracy |
| test_accuracy, test_f1, test_auc | held-out, problem-disjoint |
| threshold | the decision threshold shipped in `server/src/predict.rs` |
| train_submissions / train_problems | split size |
| test_submissions / test_problems | split size |

**T12 — Routing error decomposition.** *Source: E9.* The C5 evidence, and the table that keeps the
model honest.
| Column | Meaning |
|---|---|
| model_set | original / retrained / +allocation features |
| misroute_cpp, misroute_java, misroute_python, misroute_c | genuinely-Heavy routed to Low, per language |
| misroute_all | aggregate |
| routed_high_pct | share sent to the High tier (the cost side) |
| misroutes_under_tier | of the misroutes, how many still fit inside the Low tier's limit |
| genuine_over_limit | the remainder — the real failures |
| note | the measurement basis for the memory figure |

**T13 — Threshold sensitivity.** *Source: E9.* Shows accuracy-optimal is the wrong objective.
| Column | Meaning |
|---|---|
| threshold_set | deployed / each model's CV-optimal / misroute-minimising |
| misroute_all | resulting misroute rate |
| routed_high_pct | resulting High-tier share |
| degenerate | true if the set collapses to routing almost everything High |

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

**F7 — Model evaluation.** ROC curves for the unified and specialized models on the problem-disjoint
test split, with each shipped threshold marked, plus a secondary panel plotting misroute rate against
the share routed High as the threshold sweeps — the trade-off that shows why accuracy-optimal
thresholds are the wrong objective. Plotted from the trainer's own outputs, not re-derived.

## I.5 Traceability

Every claim → experiment → element → section. Nothing may appear in the paper without a row here.

| Claim | Experiments | Tables | Figures | Paper section |
|---|---|---|---|---|
| C1 | E1 | T1 | F2 | Motivation / workload characterisation |
| C2 | E4, E6 | T3 | F4 | Mechanism and safety |
| C3 | E2, E3 | T2, T5 | F3 | Results |
| C4 | E5 | T4 | — | Prediction analysis |
| C5 | E9 | T4 | — | Predictive model and its training data |
| — | E7 | supplementary | — | Boundary / limitations |
| — | E8 | not tabled | — | Deployment |

**Every table and figure is now accounted for.** T1–T5 and F1–F4 are the paper; everything else in I.3
and I.4 is either merged into them or listed as cut in I.6. If an experiment produces data that does not
land in one of these nine elements, that data belongs in supplementary material, not in the paper.

## I.6 Six-page element budget

**The page limit is the binding constraint.** A 6-page two-column IEEE paper holds roughly 4.5 pages of
body text plus about one page of tables and figures, so at most **5 tables and 4 figures** can survive.
Thirteen tables and seven figures cannot. The budget below is therefore part of the design, not a
post-hoc cut: **the harness should emit the merged shapes directly**, so nothing has to be assembled by
hand at submission time.

### The five tables that ship

| ID | Title | Merged from | Source | Carries |
|---|---|---|---|---|
| **T1** | Corpus and demand distribution | T1 + T2 | E1 | C1 |
| **T2** | Configuration comparison | T3 + T4 | E2, E3 | C3 |
| **T3** | Correctness envelope of promotion | T5 | E4, E6 | C2 |
| **T4** | Predictive model: corpus, performance, routing | T10 + T11 + T12 + T13 | E9 | C5 |
| **T5** | Headline summary | T9 | all | C1–C5 |

**T1** merges the corpus composition and the demand percentiles into one table: the corpus description
becomes a caption line, and the body is the percentile table with the sub-25/256/1024/2048 shares.
**T2** merges the static sweep and the policy comparison, because they answer the same question from two
sides; feasibility (`mem_failures == 0`) becomes the column that separates valid cells from invalid ones.
**T4** merges four tables into one, because a model's performance is meaningless without its corpus and
its error decomposition — they are one argument, not three.

### The four figures that ship

| ID | Title | Source | Carries | Notes |
|---|---|---|---|---|
| **F1** | System architecture | static | context | half-column, compact |
| **F2** | Demand distribution | E1 | C1 | the single most important figure |
| **F3** | Tension frontier | E2, E3 | C3 | throughput vs memory-caused failures, adaptive marked |
| **F4** | Promotion safety | E4, E6 | C2 | latency distribution + the allocation-rate race threshold |

### What is cut, and where it goes

| Element | Why cut | Where it goes |
|---|---|---|
| T8 cloud provisioning | E8 may not run, and the numbers are derived rather than measured | a short paragraph in the deployment section, or the journal version |
| F5 promotion latency (standalone) | folded into F4 | — |
| F6 burst response | cannot be driven end-to-end without E3; a simulated burst is not publishable | dropped unless E3 makes it measurable, in which case it displaces F4 |
| F7 model evaluation (ROC) | the AUC/F1 numbers live in T4; the ROC curves are the least informative panel | the trade-off panel (misroute vs share routed High) survives as a small inset in T4 |
| Per-language breakdowns | T4 carries per-language misroute in one column group | supplementary material |
| Threshold sensitivity (standalone) | three rows; fits as a T4 sub-block | — |

### Page allocation

| Section | Pages |
|---|---|
| Title, abstract, index terms | 0.25 |
| Introduction and motivation | 0.75 |
| Theoretical model | 0.75 |
| System implementation | 0.75 |
| Results (T1–T4, F1–F4) | 2.25 |
| Deployment and cost | 0.4 |
| Boundary analysis and limitations | 0.35 |
| Conclusion and references | 0.5 |

### Consequences for the experiments

1. **T2 is the expensive table and the one that must be right.** It carries C3, and it is the only table
   where the static sweep and the adaptive policies meet. Every cell must come from a real run.
2. **F3 must be plotted from E2 and E3 together**, or the frontier has nothing to mark adaptive against.
3. **T4 must be emitted by E9 as one table**, not assembled from four training reports.
4. **Nothing may be added to the paper without removing something.** If a fifth figure is needed, one of
   F1/F4 goes — F1 is the least load-bearing, since the mechanism is describable in prose.
5. **Supplementary material is the overflow valve.** Per-language detail, the full 30-cell sweep and the
   ROC curves belong there, and the paper should say so explicitly rather than compress them to illegibility.

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

Consequence: at 16 × 256 MiB = 4 GiB against the `e2-standard-4`'s usable budget, the instance sits at
roughly a quarter of its RAM and is **never under memory pressure at any load**. Arrivals queue on the
semaphore, not on memory. The scenario the thesis is about cannot currently occur. This is why **E3
requires a code change**, and why every queueing number produced so far was simulated.

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

**Paper element.** T1 (merged), F2.

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

**Paper element.** T2 (merged), F3.

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

**Paper element.** T2 (merged), F3, and the T5 headline row for C3.

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

**Paper element.** T3 (merged), F4.

**Kill condition.** Any case that fails under `reactive`, passes under `baseline`, has a memory-caused
verdict and a non-zero `oom_kill` delta → C2 is false. Report the rate threshold either way; if
promotion only fails above a high rate, that is a bounded, statable limitation.

**Effort.** Medium, but delicate — repeated trials are required because the monitor is asynchronous.

## E5 — Is prediction inherently insufficient? → **C4**, T6

**Question.** Is predictive-only worse because of *this model*, or because of *prediction*?

**Hypothesis.** Even with perfect prediction, predictive-only cannot match reactive, because it must
reserve worst-case for everything it flags.

**Procedure.** Three arms on the same corpus (see **E9** for what the shipped classifier is and how it
was validated):
1. `predictive` with the shipped classifier;
2. `predictive` with an **oracle label** (the true peak from E1) — the best possible predictor;
3. `reactive`.

For arms 1–2, disable reactive recovery so the prediction is the only defence.

**Controls.** The oracle arm is the control. Any gap between arm 2 and arm 3 is a property of
prediction, not of the model.

**Metrics.** False-heavy rate (wasted reservation), false-light rate (correctness risk), reservation
held, and the reservation floor prediction cannot go below.

**Paper element.** T4 (merged) — the oracle arm's reservation floor is a row in the model table.

**Kill condition.** If the oracle arm matches reactive, C4 is false and the honest conclusion is
"this classifier is under-trained", not "prediction is insufficient". Report either outcome.

**Effort.** Medium. Requires E1's true peaks as labels, and **must follow E9** — the C4 argument only
holds once the model's own quality has been established, otherwise "prediction is insufficient" is
indistinguishable from "this model is under-trained".

## E6 — Mechanism verification at the kernel → supports **C2**, T5

**Question.** Does the promotion trigger fire exactly at the configured watermark, and is the cgroup
write the promotion?

**Procedure.**
1. Reproduce the boundary cases: 170 MiB must **not** promote; 185 MiB **must**.
2. Sample `memory.max` / `memory.high` / `memory.events` before, during and after.
3. Assert `memory.max` goes `268435456` → `max` and that the judge's `tier_promoted` agrees.

**Controls.** **Both directions are required** — a non-promoting policy at the same demand must fail,
and a promoting policy must succeed. One direction alone proves nothing.

**Paper element.** T3 (mechanism rows); supports F4.

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

**Paper element.** Supplementary — the tier floor is stated in the limitations section rather than tabled.

**Kill condition.** If no language fails at 128 MiB for a structural reason, the floor claim is wrong
and the achievable ratio is 16×, not 8×.

**Effort.** Small.

## E8 — Multi-VM scaling → T8 (only if the single-VM result holds)

**Question.** Does the advantage survive when admission spans more than one VM?

**Procedure.** The `e2-standard-4` in `asia-south1` is the testbed for every experiment, so E1–E7 run on
it directly. E8 adds a **second VM of the same shape** behind a dispatcher that computes admission from
aggregate reservation, and repeats the E2/E3 matrix across the pair.

**Controls.** Same instance shape, corpus, schedule and configuration as the single-VM runs, so the only
difference is the number of VMs.

**Metrics.** Sustained rate, correctness, reservation, coordination overhead.

**Paper element.** Not tabled. Cloud cost is a short paragraph in the deployment section; any derived
figure is marked `D`.

**Kill condition.** If the advantage vanishes with coordination, state the result as single-VM only.

**Effort.** Large, and the only experiment that provisions more than one billable instance. Record the
hourly and monthly figure and the teardown command in the paper's deployment section.

## E9 — Predictive model: training data, evaluation and deployment → **C5**, T10–T13, F7

The classifier is half the design, and the paper currently has no element that describes what it was
trained on, how it was validated, or how its errors decompose. This experiment produces that, and it is
a prerequisite for E5 — you cannot argue that prediction is insufficient without first establishing that
the prediction is as good as it can reasonably be.

### E9.1 Feature extraction (fixed; do not change without re-running everything)

The judge does not load a model at run time. The trained model is **compiled into the Rust binary**, so
the feature vector is a compile-time contract with four independent sites that must agree:

| Site | What it pins |
|---|---|
| `feature-extraction-pipeline/src/features.rs` | the 26 base AST features |
| `model-training/train_advanced_xgboost.py` (`get_feature_cols`) | 26 base + 10 engineered |
| `model-training/regenerate_models.sh` | the expected vector length |
| `server/src/predict.rs` | the order the judge builds the vector in |

**26 base AST features:** nesting_depth, max_loop_depth, total_loops, cyclomatic_complexity,
is_recursive, recursive_call_count, large_alloc_flag, alloc_size_max, alloc_size_total, alloc_sites,
alloc_unknown_sites, has_fast_io, has_heavy_datastructure, has_modulo_arithmetic, has_bitmask_ops,
has_graph_adjacency, total_functions, total_calls, total_subscripts, total_2d_subscripts,
total_arithmetic_ops, max_integer_constant, ast_node_count, ast_depth, source_loc, source_chars.

**10 engineered:** loop_density, call_density, subscript_density, branch_density, arithmetic_density,
subscript_2d_ratio, recursion_intensity, log_max_constant, log_ast_nodes, log_source_chars.

**4 language one-hots** (unified model only): lang_C, lang_C++, lang_Java, lang_Python.

So the specialized models take **36** features and the unified model takes **40**. A mismatch between
these sites produces *silently wrong scores*, not an error — which is why the plan requires a length
assertion at each site and a test that the four agree.

### E9.2 Training corpus

Two data eras exist and they are not interchangeable. The label basis is the thing that matters:

| Era | Source | Size | Unique problems | Label rule | Label basis |
|---|---|---|---|---|---|
| CodeNet | `iNeil77/CodeNet` parquet | 164,686 submissions (Light 100,000 / Heavy 64,686) | 2,520 | Light < 25 MiB; Heavy ≥ 100 MiB; the 25–100 MiB band **dropped** | **measured** memory |
| CodeContests | `deepmind/code_contests` | 30,000 files (5,000 Light / 5,000 Heavy per language) | 148 | Light if `difficulty ≤ 3 or len(code) < 650`; Heavy if `difficulty ≥ 5 or len(code) ≥ 1200` | **length heuristic** |

The label change was necessary and the paper must say why: over a 12.7M-row CodeNet scan, source length
explains only **7–18%** of the variance of measured memory (Python r = 0.4278, C++ r = 0.3648,
C r = 0.3107, Java r = 0.2615). A length threshold is a weak proxy for the thing being predicted, and no
program in the CodeContests corpus exceeds 50 MiB measured, so its "Heavy" class is not what the tier's
limit actually tests.

**Decision (adopted): CodeNet is the training corpus.** CodeContests is retained only as the historical
era and is never used to train a model the paper reports.

### E9.3 Training and validation protocol

- **Split:** 80/20 `GroupShuffleSplit` **grouped by problem**, seed 42 → 125,159 train submissions over
  1,930 problems, 28,687 test submissions over 483 problems. Grouping is what makes the test measure
  generalisation to *unseen problems* rather than memorisation.
- **Cross-validation:** `GroupKFold` on the training split, again grouped by problem.
- **Class imbalance:** `scale_pos_weight = negatives / positives`, computed per model.
- **Threshold selection:** Youden's J (sensitivity + specificity − 1) from the CV ROC curve, clamped to
  [0.2, 0.8] to prevent degenerate edges.
- **Models:** one unified multi-language model (40 features) plus specialized C++, Java, Python and C
  models (36 features).
- **Filtering:** parse-error rows are removed before training (10,840 rows, 6.6%, on CodeNet).

### E9.4 Evaluation — misroute, not accuracy

**Accuracy is the wrong metric.** The Light majority dominates it, so a model that routes everything Low
scores well and is dangerous. The metric that matters is the **misroute rate**: a genuinely-Heavy program
sent to the Low tier.

**And misroute alone is also the wrong objective**, because it is minimised by routing everything High.
The paper must report **misroute against the share routed High** — the trade-off — not misroute alone.

**The error decomposition is required.** A "Heavy" label means ≥ 100 MiB, but the Low tier's hard limit
is 256 MiB. A program measured between those two numbers is labelled Heavy, counted as a misroute, and
yet completes inside Low anyway. So the raw misroute rate must be split into genuine over-limit failures
and boundary artefacts, with the measurement basis stated.

### E9.5 Procedure

1. Rebuild the corpus with `extract_codenet.py` at a fixed seed; record the manifest hash.
2. Extract features with the Rust extractor; assert the vector length matches at all four sites.
3. Train with `train_advanced_xgboost.py`; record the model parameters, seed and device.
4. Run `evaluate_routing.py` through the **deployed** thresholds, not the trainer's own.
5. Produce T10–T13 and F7.
6. Run `regenerate_models.sh` and confirm the judge binary rebuilds and its routing matches the
   Python evaluation on a sample of submissions.

### E9.6 Controls

- **Leakage.** The split is grouped by problem; verify no problem appears in both sides, and verify no
  test-case, expected-output or timing signal reaches the feature vector. Feature extraction reads
  **source only**.
- **Device.** GPU and CPU runs are not bit-identical (unified AUC 0.9502 vs 0.9503) because float
  reduction order differs. Record which was used, and do not mix them within a reported table.
- **Threshold provenance.** The judge ships fixed `THRESHOLD_*` constants. State which models those
  thresholds were tuned for, because applying thresholds tuned for one model to another is exactly the
  error that produced the deployed set.
- **End-to-end agreement.** A model that scores correctly in Python but is compiled incorrectly into
  Rust is a silent failure. The paper must be able to show the judge's verdict path agrees with the
  offline evaluation.

### Paper element

**T4 (merged)** — corpus, model performance and routing errors in one table, with the
misroute-vs-share-routed-High trade-off as a small inset (F7 does not survive as a standalone figure;
see I.6). Feeds E5: the oracle arm needs the measured peaks, and the C4 argument needs the model's
quality established first.

### Kill condition

If the reported numbers cannot be reproduced from the committed corpus, manifest and seed, C5 fails and
no routing claim may be made. If the error decomposition does not survive the 256 MiB reframing — i.e.
most misroutes are genuine over-limit failures — then the routing story changes materially and must be
restated.

### Effort

Medium. One training run on the reference hardware; the expensive part is the corpus scan.


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

- **Single instance type.** All measurements come from one `e2-standard-4` shape. Cache, memory
  bandwidth and core count all move the promotion race, so the result is a statement about that shape
  rather than about cloud hardware in general. E8 tests whether it generalises to a second VM of the
  same shape; it does not test a different shape.
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
| 9 | **E9** model training and evaluation | must precede E5's argument; produces the model's own evidence |
| 10 | **E8** multi-VM scaling | only if the single-VM result holds |

**Stop conditions.**
- E1 shows demand is not bimodal → stop, re-scope the whole project.
- E2 finds one static configuration matching adaptive at equal correctness → stop, report that.
- E4 shows an unbounded promotion race → rework the mechanism before any throughput claim.

**Definition of done for the paper.** Every claim in I.2 has a table or figure in I.3/I.4; every number
carries an M/D/S label; every result directory has a `PROVENANCE.md`; the `.tex` compiles with zero
errors, zero undefined references and zero overfull boxes; and no number in the paper traces to
anything outside the repository's current tree.

---

# Part VIII — Decisions

All five are now **decided**, and the decisions are binding on the experiments above. Change one only
by changing this section.

## D1 — Admission control becomes memory-derived. **ADOPTED.**

`MAX_CONCURRENT` changes from a constant to a runtime policy: `fixed(N)` for the static sweep in E2, and
`memory_derived(reserve_mib)` for E3, which admits while `(available_ram - reserve) >= tier_reservation`.

**Why.** With a fixed 16, the instance sits at roughly a quarter of its RAM and memory never becomes the
binding constraint, so the adaptive mechanism has nothing to act on and the central claim stays analytic.
E3 does not exist without this change.

**Scope.** One module (`server/src/queue.rs`) plus configuration plumbing. The `fixed(N)` mode preserves
current behaviour exactly, so the change is not a rewrite.

**Obligation.** Every result must report which admission policy was active, because it dominates all
other effects.

## D2 — Synthetic heavy submissions are confined to mechanism experiments. **ADOPTED.**

Permitted in **T5 and F5** (the correctness envelope), where they are legitimate adversarial inputs.
**Excluded** from T1–T4 and F2/F4 (workload characterisation and results), unless T1 carries them as a
separate source row and F2 renders them distinguishably.

**Why.** They were written to trigger the watermark. Measuring how often the watermark triggers on inputs
authored to trigger it is circular, and a reviewer will say so. Used as mechanism probes they are
defensible; used as workload evidence they are not.

## D3 — Anchor the comparison to real judge configurations, without deploying one. **ADOPTED.**

E2 keeps the 30-cell static sweep, and the paper adds a short subsection stating each mainstream judge's
**documented** configuration — per-submission limit and worker model for DOMjudge, DMOJ and Judge0 — and
maps each onto the cell of the sweep that corresponds to it.

**Why.** The sweep is far better than a single worst-case strawman, but unanchored it invites "we don't
provision 2048 MiB, we do X". Mapping real configurations onto sweep cells makes the comparison concrete
at the cost of a few hours of reading. Deploying and benchmarking a real judge is not worth the effort
for a conference paper and mostly re-measures someone else's system.

## D4 — Corpus scale: decouple characterisation from the sweep. **ADOPTED.**

- **E1** uses the **full corpus** (target ≥1000 submissions across ≥100 problems) — a single pass, so it
  is cheap.
- **E2 and E3** use a **fixed subset** of that corpus, stated in the paper, with a fixed seed.

**Why.** E2 × E3 is up to 34 full passes; multiplying that by 1000 submissions on one VM is days of
billable wall-clock. The distribution statistics need the large sample; the throughput and correctness
comparisons need a stable, smaller workload. Reporting both sizes explicitly is honest and costs nothing.

**Obligation.** T3 and T4 must state the subset size, and F2 must state that it is drawn from the full
corpus.

## D5 — Six pages, five authors. **RESOLVED.**

**Page limit: 6 including references.** Authorship confirmed from the current PDF (see I.1b): Bharath
Aashish R, Dhanush M, Hemanthkumar K, Iniyaa P, and Indumathy P as Assistant Professor — five of the
template's six blocks.

**Consequence.** The 6-page limit is now the binding constraint on the entire design. I.6 sets the
element budget: **five tables and four figures**. The other eight tables and three figures are either
merged into those, cut, or moved to supplementary material. This is settled before drafting rather than
discovered at submission.

**Still to settle (not blocking):** whether the guide's affiliation wording is normalised to match the
other four authors, and whether her position in the block is intended as written.
