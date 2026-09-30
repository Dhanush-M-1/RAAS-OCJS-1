import { useEffect, useRef, useState } from "react";
import ThemeProvider from "./components/ThemeProvider";
import ThemeToggle from "./components/ThemeToggle";
import CodeEditor from "./components/CodeEditor";
import StrategyComparison from "./components/StrategyComparison";
import TestCaseChart from "./components/TestCaseChart";
import TestCaseTable from "./components/TestCaseTable";
import {
  Eyebrow,
  MetricStat,
  Section,
  TextToggle,
  VerdictText,
} from "./components/ui";
import { PROBLEMS, type CodeLanguage } from "./problems";
import {
  METRICS,
  METRIC_SHORT,
  formatAllocatedMemory,
  labelOf,
  tierLabel,
  type ComparisonMetric,
  type JudgeResult,
} from "./judge";
import { formatBytes } from "./status";

const BACKEND_URL = "http://localhost:3000";

const STRATEGIES = [
  "Baseline",
  "Predictive",
  "Reactive",
  "Hybrid",
  "Run all four strategies",
] as const;
type Strategy = (typeof STRATEGIES)[number];

const RUN_ALL = ["baseline", "predictive", "reactive", "hybrid"] as const;

const LANGS: CodeLanguage[] = ["C", "C++", "Java", "Python"];

function apiLanguage(lang: CodeLanguage): string {
  switch (lang) {
    case "C++":
      return "cpp";
    case "C":
      return "c";
    case "Java":
      return "java";
    case "Python":
      return "python";
  }
}

function monacoLanguage(lang: CodeLanguage): string {
  switch (lang) {
    case "C++":
      return "cpp";
    case "C":
      return "c";
    case "Java":
      return "java";
    case "Python":
      return "python";
  }
}

function orderStrategyResults(results: JudgeResult[]): JudgeResult[] {
  return RUN_ALL.flatMap((approach) => {
    const result = results.find(
      (candidate) => candidate.approach.toLowerCase() === approach,
    );
    return result ? [result] : [];
  });
}

type BackendStatus = "checking" | "online" | "offline";

const STATUS_LABEL: Record<BackendStatus, string> = {
  online: "online",
  offline: "offline",
  checking: "checking",
};

/** Plain mono status text. Color only carries meaning: offline is --danger. */
function BackendIndicator({ status }: { status: BackendStatus }) {
  const offline = status === "offline";
  return (
    <span
      role="status"
      title={`judge backend ${BACKEND_URL}`}
      className={`eyebrow inline-flex items-center gap-2 ${offline ? "text-danger" : "text-mute"}`}
    >
      <span
        className={`inline-block h-1.5 w-1.5 ${offline ? "bg-danger" : "bg-mute"}`}
        aria-hidden="true"
      />
      backend {STATUS_LABEL[status]}
    </span>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <JudgePage />
    </ThemeProvider>
  );
}

function JudgePage() {
  const [selectedProblemId, setSelectedProblemId] = useState<string>(
    PROBLEMS[0].id,
  );
  const [language, setLanguage] = useState<CodeLanguage>("Python");
  const [strategy, setStrategy] = useState<Strategy>("Predictive");

  const currentProblem =
    PROBLEMS.find((p) => p.id === selectedProblemId) ?? PROBLEMS[0];
  const [source, setSource] = useState<string>(currentProblem.code.Python);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [singleResult, setSingleResult] = useState<JudgeResult | null>(null);
  const [allResults, setAllResults] = useState<JudgeResult[] | null>(null);
  const [metric, setMetric] = useState<ComparisonMetric>("cpu_time_ms");
  const [status, setStatus] = useState<BackendStatus>("checking");
  const [runSeq, setRunSeq] = useState(0);
  const subCounterRef = useRef(0);

  useEffect(() => {
    const check = async () => {
      try {
        await fetch(`${BACKEND_URL}/health`, { method: "GET" });
        setStatus("online");
      } catch {
        setStatus("offline");
      }
    };
    check();
    const id = setInterval(check, 15000);
    return () => clearInterval(id);
  }, []);

  function handleSelectProblem(problemId: string) {
    const next = PROBLEMS.find((p) => p.id === problemId) ?? PROBLEMS[0];
    setSelectedProblemId(problemId);
    setSource(next.code[language]);
    setSingleResult(null);
    setAllResults(null);
  }

  function handleLanguageChange(nextLang: CodeLanguage) {
    setLanguage(nextLang);
    setSource(currentProblem.code[nextLang]);
    setSingleResult(null);
    setAllResults(null);
  }

  function handleResetCode() {
    setSource(currentProblem.code[language]);
  }

  async function submitOne(approach: string): Promise<JudgeResult> {
    subCounterRef.current += 1;
    const payload = {
      id: `sub-${subCounterRef.current}`,
      language: apiLanguage(language),
      source,
      test_cases: currentProblem.testCases,
      approach,
    };
    const res = await fetch(`${BACKEND_URL}/submit`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as JudgeResult;
  }

  async function handleRun() {
    setError(null);
    setSingleResult(null);
    setAllResults(null);
    setRunning(true);
    try {
      if (strategy === "Run all four strategies") {
        const results: JudgeResult[] = [];
        for (const s of RUN_ALL) {
          results.push(await submitOne(s));
        }
        setAllResults(orderStrategyResults(results));
        setSingleResult(null);
      } else {
        const approach =
          strategy === "Baseline" ? "baseline" : strategy.toLowerCase();
        setSingleResult(await submitOne(approach));
        setAllResults(null);
      }
      setRunSeq((n) => n + 1);
    } catch {
      setError("Unable to connect to judge backend (http://localhost:3000)");
      setSingleResult(null);
      setAllResults(null);
    } finally {
      setRunning(false);
    }
  }

  const hasResult = singleResult !== null || allResults !== null;
  const runDisabled = running || status === "offline";

  const singleSpec = singleResult
    ? [
        { label: "starting tier", value: tierLabel(singleResult.tier_started) },
        {
          label: "memory allocated",
          value: formatAllocatedMemory(singleResult),
        },
        {
          label: "memory used (peak)",
          value: formatBytes(singleResult.peak_memory_bytes),
        },
        {
          label: "tier promoted",
          value: singleResult.tier_promoted ? "yes" : "no",
        },
        {
          label: "promotion time",
          value: `${singleResult.promotion_time_ms} ms`,
        },
      ]
    : [];

  return (
    <div className="min-h-screen bg-bg text-fg">
      <header className="sticky top-0 z-20 border-b-2 border-rule bg-bg">
        <div className="mx-auto flex w-full max-w-[1680px] items-baseline justify-between gap-6 px-6 py-3 lg:px-10">
          <div className="flex min-w-0 items-baseline gap-4">
            <span className="serif-display text-[22px] text-fg">RAAS-OJS</span>
            <span className="eyebrow hidden truncate text-mute md:inline">
              resource-aware adaptive scheduling
            </span>
          </div>
          <div className="flex shrink-0 items-baseline gap-5">
            <BackendIndicator status={status} />
            <ThemeToggle />
          </div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-[1680px] px-6 py-10 lg:px-10 lg:py-14">
        <div className="grid grid-cols-1 gap-x-12 gap-y-14 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
          {/* ------------------------------------------------------ left column */}
          <div className="flex flex-col gap-14">
            <Section
              title="01 — benchmarks"
              meta={`${PROBLEMS.length} available`}
              bodyClassName="pt-1"
            >
              <ul>
                {PROBLEMS.map((prob) => {
                  const isSelected = prob.id === currentProblem.id;
                  return (
                    <li
                      key={prob.id}
                      className="border-b border-rule last:border-b-0"
                    >
                      <button
                        type="button"
                        onClick={() => handleSelectProblem(prob.id)}
                        className={`group flex w-full items-baseline gap-4 border-l-2 py-3 pl-3 pr-1 text-left transition-colors ${
                          isSelected
                            ? "border-accent-ink"
                            : "border-transparent"
                        }`}
                      >
                        <span
                          className={`eyebrow ${isSelected ? "text-accent-ink" : "text-mute"}`}
                        >
                          {String(prob.number).padStart(2, "0")}
                        </span>
                        <span
                          className={`min-w-0 flex-1 truncate text-[15px] ${
                            isSelected
                              ? "text-accent-ink"
                              : "text-fg group-hover:text-accent-ink"
                          }`}
                        >
                          {prob.title.replace(/^\d+\.\s*/, "")}
                        </span>
                        <span className="eyebrow shrink-0 text-mute">
                          {prob.category}
                        </span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            </Section>

            <Section
              title="02 — problem"
              bodyClassName="pt-4"
            >
              <div className="flex flex-wrap items-baseline gap-x-6 gap-y-2">
                <span className="eyebrow text-mute">
                  {currentProblem.category}
                </span>
                <span className="eyebrow text-mute">
                  {currentProblem.complexity}
                </span>
                <span className="eyebrow text-accent-ink sm:ml-auto">
                  {currentProblem.targetStrategy}
                </span>
              </div>
              <h1 className="serif-display mt-4 text-[34px] text-fg">
                {currentProblem.title}
              </h1>
              <p className="mt-4 max-w-prose text-[15px] leading-relaxed text-mute">
                {currentProblem.description}
              </p>
            </Section>

            <Section
              title="03 — test cases"
              meta={`${currentProblem.testCases.length} cases`}
              bodyClassName="pt-1"
            >
              {currentProblem.testCases.map((tc, i) => (
                <div
                  key={i}
                  className="grid grid-cols-1 gap-x-10 gap-y-4 border-b border-rule py-5 last:border-b-0 sm:grid-cols-2"
                >
                  <div className="min-w-0">
                    <Eyebrow>input {i + 1}</Eyebrow>
                    <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap font-mono text-[13px] leading-relaxed text-fg">
                      {tc.displayInput ??
                        (tc.input.length > 250
                          ? `${tc.input.slice(0, 200).trim()} ... [${tc.input.length.toLocaleString()} chars]`
                          : tc.input.trim() || "(empty)")}
                    </pre>
                  </div>
                  <div className="min-w-0">
                    <Eyebrow>expected output</Eyebrow>
                    <pre className="mt-2 overflow-auto whitespace-pre-wrap font-mono text-[13px] leading-relaxed text-fg">
                      {tc.expected.trim()}
                    </pre>
                  </div>
                </div>
              ))}
            </Section>
          </div>

          {/* ----------------------------------------------------- right column */}
          <div className="flex flex-col gap-14">
            <section>
              <header className="flex flex-wrap items-end justify-between gap-x-8 gap-y-4 border-b-2 border-rule pb-3">
                <div className="flex flex-wrap items-end gap-x-8 gap-y-3">
                  <label className="flex flex-col gap-2">
                    <Eyebrow>language</Eyebrow>
                    <select
                      value={language}
                      onChange={(e) =>
                        handleLanguageChange(e.target.value as CodeLanguage)
                      }
                      className="border-b border-rule bg-transparent py-1.5 pr-6 font-mono text-[13px] text-fg outline-none transition-colors hover:border-fg focus:border-accent"
                    >
                      {LANGS.map((l) => (
                        <option
                          key={l}
                          value={l}
                        >
                          {l}
                        </option>
                      ))}
                    </select>
                  </label>

                  <label className="flex flex-col gap-2">
                    <Eyebrow>strategy</Eyebrow>
                    <select
                      value={strategy}
                      onChange={(e) => {
                        setStrategy(e.target.value as Strategy);
                        setSingleResult(null);
                        setAllResults(null);
                      }}
                      className="border-b border-rule bg-transparent py-1.5 pr-6 font-mono text-[13px] text-fg outline-none transition-colors hover:border-fg focus:border-accent"
                    >
                      {STRATEGIES.map((s) => (
                        <option
                          key={s}
                          value={s}
                        >
                          {s}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>

                <div className="flex items-center gap-3">
                  <button
                    type="button"
                    onClick={handleResetCode}
                    title="Reset code to the problem template"
                    className="eyebrow border border-rule px-3 py-2 text-mute transition-colors hover:border-fg hover:text-fg focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                  >
                    reset code
                  </button>
                  <button
                    type="button"
                    onClick={handleRun}
                    disabled={runDisabled}
                    aria-busy={running}
                    className="eyebrow bg-accent-ink px-4 py-2 text-on-accent transition-opacity hover:opacity-90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {running ? "running…" : "run code"}
                  </button>
                </div>
              </header>

              <div className="h-[380px] border-b border-rule">
                <CodeEditor
                  language={monacoLanguage(language)}
                  value={source}
                  onChange={setSource}
                />
              </div>
            </section>

            {error ? (
              <div
                role="alert"
                className="border-l-2 border-danger pl-4"
              >
                <p className="text-sm text-danger">{error}</p>
                <p className="mt-1 text-sm text-mute">
                  Make sure the judge is running (
                  <code className="font-mono">{BACKEND_URL}</code>) and the
                  Docker runtime images are built.
                </p>
              </div>
            ) : null}

            {hasResult ? (
              <div
                key={runSeq}
                className="reveal flex flex-col gap-14"
              >
                {singleResult ? (
                  <Section
                    title="04 — results"
                    meta={labelOf(singleResult)}
                    bodyClassName="pt-6"
                  >
                    <div className="flex flex-wrap items-baseline justify-between gap-x-8 gap-y-2">
                      <VerdictText
                        verdict={singleResult.verdict}
                        className="text-base"
                      />
                      <span className="eyebrow number text-mute">
                        {singleResult.submission_id}
                      </span>
                    </div>

                    <div className="mt-6 grid grid-cols-2 gap-x-8 border-y border-rule sm:grid-cols-4">
                      <MetricStat
                        label="cpu time"
                        value={`${singleResult.cpu_time_ms} ms`}
                      />
                      <MetricStat
                        label="wall time"
                        value={`${singleResult.wall_time_ms} ms`}
                      />
                      <MetricStat
                        label="memory used"
                        value={formatBytes(singleResult.peak_memory_bytes)}
                      />
                      <MetricStat
                        label="memory allocated"
                        value={formatAllocatedMemory(singleResult)}
                        valueClassName="text-base"
                      />
                    </div>

                    {singleResult.cases.length > 0 ? (
                      <div className="mt-8">
                        <div className="flex flex-wrap items-baseline justify-between gap-x-8 gap-y-3 border-b border-rule pb-3">
                          <span className="eyebrow text-mute">
                            per test case
                          </span>
                          <TextToggle
                            options={METRICS.map((m) => ({
                              value: m,
                              label: METRIC_SHORT[m],
                            }))}
                            value={metric}
                            onChange={setMetric}
                            label="per-case metric"
                          />
                        </div>
                        <div className="pt-3">
                          <TestCaseChart
                            cases={singleResult.cases}
                            metric={metric}
                          />
                        </div>
                        <div className="pt-6">
                          <TestCaseTable
                            cases={singleResult.cases}
                            metric={metric}
                            tierStarted={singleResult.tier_started}
                          />
                        </div>
                      </div>
                    ) : null}

                    <dl className="mt-8 border-t border-rule">
                      {singleSpec.map((row) => (
                        <div
                          key={row.label}
                          className="flex items-baseline justify-between gap-8 border-b border-rule py-3"
                        >
                          <dt className="text-sm text-mute">{row.label}</dt>
                          <dd className="number font-mono text-[13px] text-fg">
                            {row.value}
                          </dd>
                        </div>
                      ))}
                    </dl>
                  </Section>
                ) : null}

                {allResults ? (
                  <Section
                    title="04 — results"
                    meta={`${allResults.length} strategies`}
                  >
                    <StrategyComparison
                      results={allResults}
                      metric={metric}
                      onMetricChange={setMetric}
                    />
                  </Section>
                ) : null}
              </div>
            ) : null}
          </div>
        </div>
      </main>
    </div>
  );
}
