export interface ChartTokens {
  bg: string;
  fg: string;
  mute: string;
  rule: string;
  accent: string;
  danger: string;
}

/**
 * Recharts needs concrete color strings, so read the live `data-theme` scoped
 * variables at render time instead of mirroring hex values here. Components
 * re-render on theme change via useTheme, which keeps charts in sync.
 */
export function chartTokens(): ChartTokens {
  const style = getComputedStyle(document.documentElement);
  const value = (name: string) => style.getPropertyValue(name).trim();
  return {
    bg: value("--bg"),
    fg: value("--fg"),
    mute: value("--mute"),
    rule: value("--rule"),
    accent: value("--accent"),
    danger: value("--danger"),
  };
}
