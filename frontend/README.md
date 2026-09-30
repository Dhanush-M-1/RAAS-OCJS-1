# RAAS-OCJS Frontend Visualizer

An interactive real-time benchmarking dashboard for evaluating Resource-Aware Adaptive Scheduling strategies on an Online Competitive Judge System.

Built with **React 19**, **TypeScript**, **Vite**, **Tailwind CSS**, and **Recharts**.

---

## Design direction

"Editorial instrument panel": the structure of a printed editorial layout (serif
headline, hairline rules, tables alongside charts) with a telemetry color
palette. There are no card boxes, no rounded bordered containers, and no pill
chips.

- **Themes**: light and dark are driven purely by CSS variables scoped to a
  `data-theme` attribute on `<html>` (default follows the OS preference; dark is
  the primary theme). Components never hardcode a color, including chart colors,
  which are read from the live variables at render time.
- **Typography**: Newsreader for headings, Geist for prose, Geist Mono for
  numbers, labels, and metadata (tabular figures throughout).
- **Color for meaning only**: `--accent` marks the single best result and the
  active/selected state; `--danger` marks failing verdicts. Everything else is
  `--fg` or `--mute`.

---

## Features

- **Problem Switcher**: switch between five competition problems (prefix sums, 0-1 knapsack 2D DP, Floyd-Warshall all-pairs shortest path, game tree search, top-K streaming frequencies).
- **Multi-Language Support**: view and run solutions in **C++**, **Python**, **Java**, and **C**.
- **Strategy Comparison**: select an individual strategy (**Baseline**, **Predictive**, **Reactive**, **Hybrid**) or execute **Run all four strategies** with a single click. The results panel shows a generated headline sentence, a strategy bar chart (the best strategy is the one accent bar), and a hairline comparison table.
- **Per-case chart and table**: CPU time, wall time, and memory switch between a per-case bar chart (failing cases in red) and a hairline table.
- **Responsive Test Case Previews**: bounded scroll previews for high-scale benchmark inputs.
- **Health Indicator**: real-time poll checking backend status on `http://localhost:3000/health`.

> **Security note:** The judge API at `http://<host>:3000` binds `0.0.0.0:3000` with **no authentication** and executes untrusted code. It must not be publicly exposed.

---

## Development & Build

### Prerequisites
- Node.js 18+ and `npm`

### Start Development Server
```bash
npm install
npm run dev
```
Access the application at `http://localhost:5173`.

### Production Build
```bash
npm run build
```
Builds optimized production assets into `dist/`.

---

## Directory Structure

```
frontend/src/
├── App.tsx                     # Main dashboard page and submission workflow
├── judge.ts                    # Judge response types and metric helpers
├── chartTokens.ts              # Reads live theme variables for Recharts
├── problems.ts                 # Competition problems, code, & generators
├── status.ts                   # Verdict failure semantics and byte formatting
├── theme.ts                    # data-theme context, storage, and system default
├── index.css                   # Theme tokens + Tailwind mapping (single source of color)
├── components/
│   ├── CodeEditor.tsx          # Theme-aware Monaco editor, tokens read at runtime
│   ├── StrategyBarChart.tsx    # Strategy comparison bars (best bar in accent)
│   ├── StrategyComparison.tsx  # Headline sentence, bar chart, and comparison table
│   ├── TestCaseChart.tsx       # Per-case bars (failing cases in danger)
│   ├── TestCaseTable.tsx       # Per-case hairline results table
│   ├── ThemeProvider.tsx       # Root theme wrapper
│   ├── ThemeToggle.tsx         # Light/dark toggle control
│   └── ui.tsx                  # Section, Eyebrow, MetricStat, TextToggle, VerdictText
```
