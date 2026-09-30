import { useMemo } from 'react'
import {
  METRICS,
  METRIC_LABELS,
  METRIC_SHORT,
  formatMetricValue,
  labelOf,
  metricValue,
  resultTierLabel,
  tierLabel,
  type ComparisonMetric,
  type JudgeResult,
} from '../judge'
import { isAcceptedVerdict } from '../status'
import StrategyBarChart from './StrategyBarChart'
import { TextToggle, VerdictText } from './ui'

const BASELINE = 'baseline'
const DASH = '\u2014'

interface StrategyComparisonProps {
  results: JudgeResult[]
  metric: ComparisonMetric
  onMetricChange: (metric: ComparisonMetric) => void
}

/**
 * Strategy comparison: a generated headline sentence, the strategy bar chart,
 * and a hairline table. The single best strategy is the only accent-colored
 * item in the chart and in the table.
 */
export default function StrategyComparison({
  results,
  metric,
  onMetricChange,
}: StrategyComparisonProps) {
  const model = useMemo(() => {
    const values = results.map((r) => metricValue(r, metric))
    let bestIdx = 0
    values.forEach((v, i) => {
      if (v < values[bestIdx]) bestIdx = i
    })
    const baselineIdx = results.findIndex((r) => r.approach.toLowerCase() === BASELINE)
    const baselineValue = baselineIdx >= 0 ? values[baselineIdx] : null
    const deltas = values.map((v, i) => {
      if (i === baselineIdx || baselineValue === null || baselineValue === 0) return null
      return ((v - baselineValue) / baselineValue) * 100
    })
    return { values, bestIdx, baselineIdx, baselineValue, deltas }
  }, [results, metric])

  const { values, bestIdx, baselineIdx, baselineValue, deltas } = model
  const best = results[bestIdx]
  const metricPhrase = metric === 'memory' ? 'peak memory' : METRIC_LABELS[metric].toLowerCase()

  let comparison: string
  if (baselineValue !== null && baselineValue > 0 && baselineIdx >= 0 && bestIdx !== baselineIdx) {
    const pct = ((values[bestIdx] - baselineValue) / baselineValue) * 100
    comparison =
      Math.abs(pct) < 0.05
        ? `${labelOf(best)} matched baseline on ${metricPhrase}.`
        : `${labelOf(best)} was ${Math.abs(pct).toFixed(1)}% ${
            pct < 0 ? 'faster' : 'slower'
          } than baseline on ${metricPhrase}.`
  } else {
    comparison = `${labelOf(best)} had the lowest ${metricPhrase}.`
  }

  const failed = results.filter((r) => !isAcceptedVerdict(r.verdict)).length
  const verdictSentence =
    failed === 0
      ? `All ${results.length} strategies passed.`
      : `${failed} of ${results.length} strategies failed.`

  return (
    <div className="flex flex-col gap-10 pt-6">
      <div>
        <p className="serif-display max-w-[46ch] text-[30px] text-fg">{comparison}</p>
        <p className="eyebrow mt-3 text-mute">{verdictSentence}</p>
      </div>

      <StrategyBarChart results={results} metric={metric} bestApproach={best.approach} />

      <div>
        <div className="flex flex-wrap items-baseline justify-between gap-x-8 gap-y-3 border-b border-rule pb-3">
          <span className="eyebrow text-mute">comparison</span>
          <TextToggle
            options={METRICS.map((m) => ({ value: m, label: METRIC_SHORT[m] }))}
            value={metric}
            onChange={onMetricChange}
            label="comparison metric"
          />
        </div>

        <div className="overflow-x-auto">
          <table className="w-full min-w-[640px] text-sm">
            <thead>
              <tr className="border-b border-rule text-left">
                <th className="eyebrow py-2 pr-4 font-normal text-mute">strategy</th>
                <th className="eyebrow py-2 pr-4 font-normal text-mute">verdict</th>
                <th className="eyebrow py-2 pr-4 font-normal text-mute">tier</th>
                <th className="eyebrow py-2 pr-4 text-right font-normal text-mute">
                  {METRIC_SHORT[metric]}
                </th>
                <th className="eyebrow py-2 pr-4 text-right font-normal text-mute">
                  delta vs baseline
                </th>
                <th className="eyebrow py-2 text-right font-normal text-mute">promotion</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-rule">
              {results.map((r, i) => {
                const isBest = i === bestIdx
                const delta = deltas[i]
                const deltaText =
                  i === baselineIdx || delta === null
                    ? DASH
                    : `${delta > 0 ? '+' : ''}${delta.toFixed(1)}%`
                const cell = isBest ? 'text-accent-ink' : 'text-fg'
                return (
                  <tr key={r.approach}>
                    <td className={`py-3 pr-4 ${cell}`}>{labelOf(r)}</td>
                    <td className="py-3 pr-4">
                      <VerdictText verdict={r.verdict} />
                    </td>
                    <td className={`py-3 pr-4 font-mono text-[13px] ${cell}`}>
                      {resultTierLabel(r)}
                    </td>
                    <td className={`number py-3 pr-4 text-right font-mono text-[13px] ${cell}`}>
                      {formatMetricValue(values[i], metric)}
                    </td>
                    <td
                      className={`number py-3 pr-4 text-right font-mono text-[13px] ${
                        isBest ? 'text-accent-ink' : 'text-mute'
                      }`}
                    >
                      {deltaText}
                    </td>
                    <td
                      className={`number py-3 text-right font-mono text-[13px] ${
                        isBest ? 'text-accent-ink' : 'text-mute'
                      }`}
                    >
                      {r.tier_promoted
                        ? `${tierLabel(r.tier_started)} \u2192 heavy \u00b7 ${r.promotion_time_ms} ms`
                        : DASH}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>

      <p className="text-sm text-mute">
        Submissions ran sequentially. Switch the metric to compare scheduling strategies.
      </p>
    </div>
  )
}
