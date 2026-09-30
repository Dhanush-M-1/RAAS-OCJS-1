/**
 * Verdict semantics.
 *
 * Color is reserved for meaning, so there is no tone palette here anymore:
 * a failing verdict renders in --danger, everything else in --fg. The accent
 * is never derived from a verdict — it marks the single best result and the
 * active/selected state only.
 */

export type Tone = 'success' | 'danger' | 'warning' | 'heavy' | 'muted'

/** The backend emits AC / WA / TLE / MLE / RE / CE / SE. */
const FAILING_VERDICTS = new Set(['WA', 'TLE', 'MLE', 'RE', 'CE', 'SE'])

export function isFailingVerdict(verdict: string): boolean {
  return FAILING_VERDICTS.has(verdict.trim().toUpperCase())
}

export function isAcceptedVerdict(verdict: string): boolean {
  return verdict.trim().toUpperCase() === 'AC'
}

export function formatBytes(bytes: number): string {
  if (!bytes) return '0 B'
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(2)} KB`
  return `${bytes} B`
}
