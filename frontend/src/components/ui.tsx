import type { ReactNode } from "react";
import { isFailingVerdict } from "../status";

/**
 * Section: an editorial block. No card box and no rounded border — the header
 * strip is closed by one heavier 2px rule, and the body is separated by
 * whitespace and hairlines supplied by the caller.
 */
export function Section({
  title,
  meta,
  actions,
  className = "",
  bodyClassName = "",
  children,
}: {
  title: string;
  meta?: ReactNode;
  actions?: ReactNode;
  className?: string;
  bodyClassName?: string;
  children: ReactNode;
}) {
  return (
    <section className={className}>
      <header className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1 border-b-2 border-rule pb-2">
        <div className="flex items-baseline gap-3">
          <h2 className="eyebrow text-mute">{title}</h2>
          {meta ? <span className="eyebrow text-mute">{meta}</span> : null}
        </div>
        {actions}
      </header>
      {children ? <div className={bodyClassName}>{children}</div> : null}
    </section>
  );
}

/** Small uppercase mono label. */
export function Eyebrow({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <span className={`eyebrow text-mute ${className}`}>{children}</span>;
}

/**
 * Verdict as plain mono text: --danger when it fails, --fg when it passes.
 * No chip, no border, no fill.
 */
export function VerdictText({
  verdict,
  className = "",
}: {
  verdict: string;
  className?: string;
}) {
  const failing = isFailingVerdict(verdict);
  return (
    <span
      className={`eyebrow ${failing ? "text-danger" : "text-fg"} ${className}`}
    >
      {verdict.trim().toUpperCase()}
    </span>
  );
}

/** A stat cell: quiet mono label above a large tabular value. */
export function MetricStat({
  label,
  value,
  valueClassName = "",
}: {
  label: string;
  value: ReactNode;
  valueClassName?: string;
}) {
  return (
    <div className="flex min-w-0 flex-col gap-2 py-3 pr-6">
      <span className="eyebrow text-mute">{label}</span>
      <span
        className={`number truncate font-mono text-2xl leading-none text-fg ${valueClassName}`}
      >
        {value}
      </span>
    </div>
  );
}

/**
 * Underlined text toggle. The active option carries a 2px accent rule, which
 * is the only accent used by this control.
 */
export function TextToggle<T extends string>({
  options,
  value,
  onChange,
  label,
}: {
  options: { value: T; label: string }[];
  value: T;
  onChange: (value: T) => void;
  label?: string;
}) {
  return (
    <div
      className="flex items-baseline gap-5"
      role="group"
      aria-label={label}
    >
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onChange(option.value)}
            aria-pressed={active}
            className={`eyebrow border-b-2 pb-1 transition-colors ${
              active
                ? "border-accent-ink text-accent-ink"
                : "border-transparent text-mute hover:text-fg"
            }`}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
