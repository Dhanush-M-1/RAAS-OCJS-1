import { useState } from "react";
import {
  Bar,
  BarChart,
  Cell,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useTheme } from "../theme";
import { chartTokens } from "../chartTokens";
import { isFailingVerdict } from "../status";
import type { CaseResult, ComparisonMetric } from "../judge";

interface CaseRow {
  name: string;
  verdict: string;
  failing: boolean;
  cpu: number;
  usedMb: number;
  allocatedMb: number;
}

interface CaseTooltipProps {
  active?: boolean;
  payload?: ReadonlyArray<{ payload: CaseRow }>;
  memory: boolean;
}

function CaseTooltip({ active, payload, memory }: CaseTooltipProps) {
  if (!active || !payload || payload.length === 0) return null;
  const row = payload[0].payload;
  return (
    <div className="border border-rule bg-bg px-3 py-2">
      <div className="eyebrow text-mute">{row.name}</div>
      <div
        className={`eyebrow mt-1 ${row.failing ? "text-danger" : "text-fg"}`}
      >
        {row.verdict}
      </div>
      <div className="eyebrow number mt-1 text-fg">
        {memory ? `${row.usedMb.toFixed(2)} MB` : `${row.cpu} ms`}
      </div>
    </div>
  );
}

interface TestCaseChartProps {
  cases: CaseResult[];
  metric: ComparisonMetric;
}

/**
 * Per-case bars. Color carries meaning only: bars for failing verdicts are
 * --danger, every other bar is --mute. No accent is used in this chart.
 */
export default function TestCaseChart({ cases, metric }: TestCaseChartProps) {
  const { theme } = useTheme();
  const tokens = chartTokens();
  const memory = metric === "memory";
  const wallFallback = metric === "wall_time_ms";
  const [reduceMotion] = useState(
    () =>
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );

  const rows: CaseRow[] = cases.map((c, i) => {
    const allocatedBytes =
      c.allocated_memory_bytes && c.allocated_memory_bytes > 0
        ? c.allocated_memory_bytes
        : 256 * 1024 * 1024;
    return {
      name: `case ${i + 1}`,
      verdict: c.verdict,
      failing: isFailingVerdict(c.verdict),
      cpu: c.cpu_time_ms,
      usedMb: c.peak_memory_bytes / (1024 * 1024),
      allocatedMb: allocatedBytes / (1024 * 1024),
    };
  });

  const barFill = (row: CaseRow) => (row.failing ? tokens.danger : tokens.mute);

  return (
    <div
      className="flex flex-col gap-3"
      data-chart-theme={theme}
      aria-label={`${memory ? "memory" : "cpu time"} per test case`}
    >
      <div className="flex items-center justify-end gap-5">
        {memory ? (
          <span className="eyebrow flex items-center gap-2 text-mute">
            <span className="inline-block h-2 w-2 border border-mute" />
            allocated
          </span>
        ) : null}
        <span className="eyebrow flex items-center gap-2 text-mute">
          <span className="inline-block h-2 w-2 bg-mute" />
          case
        </span>
        <span className="eyebrow flex items-center gap-2 text-mute">
          <span className="inline-block h-2 w-2 bg-danger" />
          failing
        </span>
      </div>

      <div className="h-[200px] w-full">
        <ResponsiveContainer
          width="100%"
          height="100%"
        >
          <BarChart
            data={rows}
            margin={{ top: 12, right: 8, bottom: 0, left: 0 }}
            barCategoryGap="30%"
          >
            <CartesianGrid
              vertical={false}
              stroke={tokens.rule}
            />
            <XAxis
              dataKey="name"
              interval={0}
              tickLine={false}
              axisLine={{ stroke: tokens.rule }}
              tick={{ fill: tokens.fg, fontSize: 10 }}
              tickMargin={8}
            />
            <YAxis
              width={64}
              tickLine={false}
              axisLine={{ stroke: tokens.rule }}
              tick={{ fill: tokens.mute, fontSize: 10 }}
              tickFormatter={(value: number) =>
                memory ? `${value} MB` : `${value} ms`
              }
            />
            <Tooltip
              cursor={{ fill: tokens.rule, opacity: 0.3 }}
              content={<CaseTooltip memory={memory} />}
            />
            {memory ? (
              <>
                <Bar
                  dataKey="allocatedMb"
                  name="allocated"
                  fill={tokens.rule}
                  stroke={tokens.mute}
                  radius={0}
                  maxBarSize={36}
                  isAnimationActive={!reduceMotion}
                />
                <Bar
                  dataKey="usedMb"
                  name="used"
                  radius={0}
                  maxBarSize={36}
                  isAnimationActive={!reduceMotion}
                >
                  {rows.map((row) => (
                    <Cell
                      key={row.name}
                      fill={barFill(row)}
                    />
                  ))}
                </Bar>
              </>
            ) : (
              <Bar
                dataKey="cpu"
                radius={0}
                maxBarSize={48}
                isAnimationActive={!reduceMotion}
              >
                {rows.map((row) => (
                  <Cell
                    key={row.name}
                    fill={barFill(row)}
                  />
                ))}
              </Bar>
            )}
          </BarChart>
        </ResponsiveContainer>
      </div>

      {wallFallback ? (
        <p className="eyebrow text-mute">
          per-case wall time is unavailable from the backend; showing cpu time
        </p>
      ) : null}
    </div>
  );
}
