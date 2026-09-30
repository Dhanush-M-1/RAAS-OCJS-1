import { useEffect, useRef, useState, type ComponentProps } from "react";
import Editor from "@monaco-editor/react";
import { useTheme } from "../theme";

// Derive the Monaco namespace type from the editor's own prop types so we
// don't need a direct `monaco-editor` dependency (it is loaded at runtime).
type BeforeMount = NonNullable<ComponentProps<typeof Editor>["beforeMount"]>;
type Monaco = Parameters<BeforeMount>[0];

interface EditorTokens {
  bg: string;
  fg: string;
  mute: string;
  rule: string;
  accent: string;
  str: string;
  mono: string;
}

/**
 * Read the live theme tokens. Colors live only in src/index.css; this component
 * reads them at runtime so switching `data-theme` re-themes the editor too.
 */
function readTokens(): EditorTokens {
  const style = getComputedStyle(document.documentElement);
  const value = (name: string) => style.getPropertyValue(name).trim();
  return {
    bg: value("--bg"),
    fg: value("--fg"),
    mute: value("--mute"),
    rule: value("--rule"),
    accent: value("--accent"),
    str: value("--syntax-string"),
    mono: value("--font-mono"),
  };
}

/** Monaco only understands `#`-prefixed hex (anything else becomes pure red),
 *  so normalise each token to bare `RRGGBB`, falling back to another token. */
function ensureHex(value: string, fallback: string): string {
  const raw = value.startsWith("#") ? value.slice(1) : value;
  if (/^[0-9a-fA-F]{6}$/.test(raw)) return raw.toLowerCase();
  const fb = fallback.startsWith("#") ? fallback.slice(1) : fallback;
  if (/^[0-9a-fA-F]{6}$/.test(fb)) return fb.toLowerCase();
  return raw.toLowerCase();
}

/** `#RRGGBBAA` — the only alpha format Monaco's hex parser accepts. */
function withAlpha(hex6: string, alpha: number): string {
  const a = Math.round(Math.min(1, Math.max(0, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `#${hex6}${a}`;
}

/**
 * Restrained palette: code in --fg, comments --mute, keywords --accent,
 * strings one muted green. Every highlight surface (current line, word and
 * bracket occurrences, find matches, indent guides, scrollbars) resolves to a
 * muted neutral so nothing paints the editor red.
 */
function defineThemes(monaco: Monaco, tokens: EditorTokens, dark: boolean) {
  const bg = ensureHex(tokens.bg, tokens.fg);
  const fg = ensureHex(tokens.fg, tokens.bg);
  const mute = ensureHex(tokens.mute, tokens.fg);
  const rule = ensureHex(tokens.rule, tokens.mute);
  const accent = ensureHex(tokens.accent, tokens.fg);
  const str = ensureHex(tokens.str, tokens.fg);

  monaco.editor.defineTheme(dark ? "raas-dark" : "raas-light", {
    base: dark ? "vs-dark" : "vs",
    inherit: true,
    rules: [
      { token: "", foreground: fg, background: bg },
      { token: "comment", foreground: mute, fontStyle: "italic" },
      { token: "keyword", foreground: accent },
      { token: "number", foreground: fg },
      { token: "string", foreground: str },
      { token: "type", foreground: fg },
      { token: "variable", foreground: fg },
    ],
    colors: {
      "editor.background": `#${bg}`,
      "editor.foreground": `#${fg}`,
      "editorGutter.background": `#${bg}`,
      // No current-line or range highlighting at all.
      "editor.lineHighlightBackground": withAlpha(rule, 0),
      "editor.lineHighlightBorder": withAlpha(rule, 0),
      "editor.rangeHighlightBackground": withAlpha(rule, 0),
      // Muted grey for selection and every other highlight.
      "editor.selectionBackground": withAlpha(mute, dark ? 0.3 : 0.24),
      "editor.inactiveSelectionBackground": withAlpha(mute, dark ? 0.16 : 0.12),
      "editor.selectionHighlightBackground": withAlpha(mute, dark ? 0.2 : 0.16),
      "editor.selectionHighlightBorder": withAlpha(mute, 0),
      "editor.wordHighlightBackground": withAlpha(mute, dark ? 0.2 : 0.16),
      "editor.wordHighlightStrongBackground": withAlpha(mute, dark ? 0.24 : 0.2),
      "editor.findMatchBackground": withAlpha(mute, dark ? 0.35 : 0.3),
      "editor.findMatchHighlightBackground": withAlpha(mute, dark ? 0.24 : 0.2),
      "editor.findRangeHighlightBackground": withAlpha(rule, dark ? 0.3 : 0.35),
      "editor.hoverHighlightBackground": withAlpha(rule, 0.3),
      "editorBracketMatch.background": withAlpha(rule, 0.5),
      "editorBracketMatch.border": withAlpha(rule, 0),
      "editorLineNumber.foreground": `#${mute}`,
      "editorLineNumber.activeForeground": `#${fg}`,
      "editorCursor.foreground": `#${accent}`,
      "editorWidget.background": `#${bg}`,
      "editorWidget.border": `#${rule}`,
      "editorIndentGuide.background1": withAlpha(rule, 0.6),
      "scrollbarSlider.background": withAlpha(rule, 0.6),
      "scrollbarSlider.hoverBackground": `#${rule}`,
    },
  });
}

interface CodeEditorProps {
  language: string;
  value: string;
  onChange: (value: string) => void;
}

export default function CodeEditor({ language, value, onChange }: CodeEditorProps) {
  const { theme } = useTheme();
  const [usingTextarea, setUsingTextarea] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const monacoRef = useRef<Monaco | null>(null);

  useEffect(() => {
    // If Monaco hasn't mounted (e.g. CDN unreachable), fall back to a textarea.
    timer.current = setTimeout(() => setUsingTextarea(true), 6000);
    return () => {
      if (timer.current) clearTimeout(timer.current);
    };
  }, []);

  // Re-read tokens and re-apply the Monaco theme whenever the theme flips.
  useEffect(() => {
    const monaco = monacoRef.current;
    if (!monaco) return;
    const dark = theme === "dark";
    defineThemes(monaco, readTokens(), dark);
    monaco.editor.setTheme(dark ? "raas-dark" : "raas-light");
  }, [theme]);

  const fallback = (
    <textarea
      className="h-full w-full resize-none bg-bg p-4 font-mono text-[13px] leading-relaxed text-fg outline-none"
      spellCheck={false}
      aria-label="Code editor"
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );

  if (usingTextarea) return fallback;

  return (
    <Editor
      height="100%"
      language={language}
      theme={theme === "dark" ? "raas-dark" : "raas-light"}
      value={value}
      onChange={(v) => onChange(v ?? "")}
      loading={fallback}
      beforeMount={(monaco) => {
        monacoRef.current = monaco;
        defineThemes(monaco, readTokens(), theme === "dark");
      }}
      onMount={() => {
        if (timer.current) clearTimeout(timer.current);
      }}
      options={{
        fontSize: 13,
        fontFamily: readTokens().mono || undefined,
        fontLigatures: false,
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 4,
        padding: { top: 16, bottom: 16 },
        // No line highlight, no occurrence/selection highlights at all.
        renderLineHighlight: "none",
        occurrencesHighlight: "off",
        selectionHighlight: false,
        roundedSelection: false,
        lineNumbersMinChars: 3,
        scrollbar: { verticalScrollbarSize: 10, horizontalScrollbarSize: 10 },
      }}
    />
  );
}
