import { useState } from "react";
import {
  Bar,
  BarChart,
  Cell,
  CartesianGrid,
  LabelList,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useTheme } from "../theme";
import { chartTokens } from "../chartTokens";
import {
  METRIC_LABELS,
  formatMetricValue,
  labelOf,
  metricValue,
  tierLabel,
  type ComparisonMetric,
  type JudgeResult,
} from "../judge";

interface StrategyRow {
  name: string;
  verdict: string;
  tier: string;
  value: number;
  usedMb: number;
  allocatedMb: number;
  isBest: boolean;
  valueText: string;
}

interface TooltipProps {
  active?: boolean;
  payload?: ReadonlyArray<{ payload: StrategyRow }>;
  metric: ComparisonMetric;
}

function StrategyTooltip({ active, payload, metric }: TooltipProps) {
  if (!active || !payload || payload.length === 0) return null;
  const row = payload[0].payload;
  return (
    <div className="border border-rule bg-bg px-3 py-2">
      <div className="eyebrow text-mute">{row.name}</div>
      <div className="eyebrow mt-1 text-fg">{row.verdict}</div>
      <div className="eyebrow mt-1 text-mute">{row.tier}</div>
      <div className="eyebrow number mt-1 text-fg">
        {metric === "memory"
          ? `${row.usedMb.toFixed(2)} MB used`
          : row.valueText}
      </div>
    </div>
  );
}

interface StrategyBarChartProps {
  results: JudgeResult[];
  metric: ComparisonMetric;
  bestApproach: string;
}

/**
 * Strategy comparison bars. Color carries meaning only: the single best
 * strategy is the one accent bar, every other bar is --mute.
 */
export default function StrategyBarChart({
  results,
  metric,
  bestApproach,
}: StrategyBarChartProps) {
  const { theme } = useTheme();
  const tokens = chartTokens();
  const memory = metric === "memory";
  const [reduceMotion] = useState(
    () =>
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );

  const rows: StrategyRow[] = results.map((r) => {
    const used = r.peak_memory_bytes;
    const allocated =
      r.allocated_memory_bytes && r.allocated_memory_bytes > 0
        ? r.allocated_memory_bytes
        : 256 * 1024 * 1024;
    const value = metricValue(r, metric);
    return {
      name: labelOf(r),
      verdict: r.verdict,
      tier: r.tier_promoted
        ? `${tierLabel(r.tier_started)} \u2192 Heavy`
        : tierLabel(r.tier_started),
      value,
      usedMb: used / (1024 * 1024),
      allocatedMb: allocated / (1024 * 1024),
      isBest: r.approach === bestApproach,
      valueText: formatMetricValue(value, metric),
    };
  });

  return (
    <div
      className="flex flex-col gap-3"
      data-chart-theme={theme}
      aria-label={`${METRIC_LABELS[metric]} across strategies`}
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
          strategy
        </span>
        <span className="eyebrow flex items-center gap-2 text-mute">
          <span className="inline-block h-2 w-2 bg-accent" />
          best
        </span>
      </div>

      <div className="h-[240px] w-full">
        <ResponsiveContainer
          width="100%"
          height="100%"
        >
          <BarChart
            data={rows}
            margin={{ top: 16, right: 8, bottom: 0, left: 0 }}
            barCategoryGap="24%"
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
              tick={{ fill: tokens.fg, fontSize: 11 }}
              tickMargin={8}
            />
            <YAxis
              width={72}
              tickLine={false}
              axisLine={{ stroke: tokens.rule }}
              tick={{ fill: tokens.mute, fontSize: 10 }}
              tickFormatter={(value: number) =>
                memory ? `${value} MB` : formatMetricValue(value, metric)
              }
            />
            <Tooltip
              cursor={{ fill: tokens.rule, opacity: 0.3 }}
              content={<StrategyTooltip metric={metric} />}
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
                      fill={row.isBest ? tokens.accent : tokens.mute}
                    />
                  ))}
                  <LabelList
                    dataKey="usedMb"
                    position="top"
                    formatter={(value) => `${Number(value).toFixed(1)}`}
                    fill={tokens.mute}
                    fontSize={10}
                  />
                </Bar>
              </>
            ) : (
              <Bar
                dataKey="value"
                radius={0}
                maxBarSize={64}
                isAnimationActive={!reduceMotion}
              >
                {rows.map((row) => (
                  <Cell
                    key={row.name}
                    fill={row.isBest ? tokens.accent : tokens.mute}
                  />
                ))}
                <LabelList
                  dataKey="value"
                  position="top"
                  formatter={(value) =>
                    formatMetricValue(Number(value), metric)
                  }
                  fill={tokens.mute}
                  fontSize={10}
                />
              </Bar>
            )}
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
