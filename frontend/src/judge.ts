import { formatBytes } from "./status";

export interface CaseResult {
  verdict: string;
  cpu_time_ms: number;
  peak_memory_bytes: number;
  allocated_memory_bytes?: number;
}

export interface JudgeResult {
  submission_id: string;
  approach: string;
  verdict: string;
  cpu_time_ms: number;
  peak_memory_bytes: number;
  allocated_memory_bytes?: number;
  wall_time_ms: number;
  tier_started: string;
  tier_promoted: boolean;
  promotion_time_ms: number;
  cases: CaseResult[];
}

export type ComparisonMetric = "cpu_time_ms" | "wall_time_ms" | "memory";

export const METRICS: ComparisonMetric[] = [
  "cpu_time_ms",
  "wall_time_ms",
  "memory",
];

export const METRIC_SHORT: Record<ComparisonMetric, string> = {
  cpu_time_ms: "CPU",
  wall_time_ms: "Wall",
  memory: "Memory",
};

export const METRIC_LABELS: Record<ComparisonMetric, string> = {
  cpu_time_ms: "CPU time",
  wall_time_ms: "Wall time",
  memory: "Memory",
};

export function tierLabel(tier: string): string {
  return tier.toLowerCase() === "low" ? "Light" : "Heavy";
}

/** Promotion is always Light -> Heavy in this system. */
export function resultTierLabel(result: JudgeResult): string {
  return result.tier_promoted
    ? `${tierLabel(result.tier_started)} \u2192 Heavy`
    : tierLabel(result.tier_started);
}

export function getAllocatedMemory(result: JudgeResult): number {
  if (
    result.allocated_memory_bytes !== undefined &&
    result.allocated_memory_bytes > 0
  ) {
    return result.allocated_memory_bytes;
  }
  if (result.tier_started === "low" && !result.tier_promoted) {
    return 256 * 1024 * 1024;
  }
  return 0;
}

export function formatAllocatedMemory(result: JudgeResult): string {
  if (result.tier_promoted) return "256 MB \u2192 uncapped (promoted)";
  const bytes = getAllocatedMemory(result);
  if (bytes === 0 || result.tier_started === "high") return "Uncapped (host)";
  return formatBytes(bytes);
}

/** The value a comparison metric reads for one strategy result. */
export function metricValue(
  result: JudgeResult,
  metric: ComparisonMetric,
): number {
  return metric === "memory"
    ? result.peak_memory_bytes
    : (result[metric] as number);
}

export function formatMetricValue(
  value: number,
  metric: ComparisonMetric,
): string {
  return metric === "memory" ? formatBytes(value) : `${value} ms`;
}

export function labelOf(result: JudgeResult): string {
  const a = result.approach.trim();
  return a.charAt(0).toUpperCase() + a.slice(1).toLowerCase();
}
