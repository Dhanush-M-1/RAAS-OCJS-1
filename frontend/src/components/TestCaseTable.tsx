import { formatBytes } from '../status'
import type { CaseResult, ComparisonMetric } from '../judge'
import { VerdictText } from './ui'

interface TestCaseTableProps {
  cases: CaseResult[]
  metric: ComparisonMetric
  tierStarted: string
}

/**
 * Per-case results as a hairline table. No bars, no cell borders, no zebra
 * striping — dividers only.
 */
export default function TestCaseTable({ cases, metric, tierStarted }: TestCaseTableProps) {
  const memoryMode = metric === 'memory'
  const wallFallback = metric === 'wall_time_ms'

  return (
    <div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[420px] text-sm">
          <thead>
            <tr className="border-b border-rule text-left">
              <th className="eyebrow py-2 pr-4 font-normal text-mute">case</th>
              <th className="eyebrow py-2 pr-4 font-normal text-mute">verdict</th>
              {memoryMode ? (
                <th className="eyebrow py-2 pr-4 text-right font-normal text-mute">allocated</th>
              ) : null}
              <th className="eyebrow py-2 text-right font-normal text-mute">
                {memoryMode ? 'peak memory' : 'cpu time'}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-rule">
            {cases.map((c, i) => (
              <tr key={i}>
                <td className="number py-3 pr-4 font-mono text-mute">
                  {String(i + 1).padStart(2, '0')}
                </td>
                <td className="py-3 pr-4">
                  <VerdictText verdict={c.verdict} />
                </td>
                {memoryMode ? (
                  <td className="number py-3 pr-4 text-right font-mono text-mute">
                    {c.allocated_memory_bytes && c.allocated_memory_bytes > 0
                      ? formatBytes(c.allocated_memory_bytes)
                      : tierStarted === 'high'
                        ? 'uncapped'
                        : '256 MB'}
                  </td>
                ) : null}
                <td className="number py-3 text-right font-mono text-fg">
                  {memoryMode ? formatBytes(c.peak_memory_bytes) : `${c.cpu_time_ms} ms`}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {wallFallback ? (
        <p className="eyebrow mt-3 text-mute">
          per-case wall time is unavailable from the backend; showing cpu time
        </p>
      ) : null}
    </div>
  )
}
