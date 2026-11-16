# T9 spike: CodeMirror 6 in the Tauri webview (disposable, v0.2)

Isolated Vite project used to measure the editor inside the real webview before the editor
feature (T16). It is **not** part of the app build: the app never imports it, Tailwind skips it
(`@source not` in `src/index.css`) and it has its own `vite.config.ts`. Results and the decision
live in `.specs/features/quire-v0-2/summary.md` (Medições and Journal, T9). Delete this folder
and the `spike-editor:*` scripts once T16 lands and the author ran the manual script below.

The document is the synthetic spec of the v0.1 spike (`../spike/generate.ts`, ~40 KB per unit
of scale). `editor.ts` is the chunk the app would load on demand; `lean.ts` is the markdown
language built straight on `@lezer/markdown` (GFM), without the HTML/CSS/JS sub-languages that
`@codemirror/lang-markdown` pulls in.

## Run

From `apps/desktop`:

```bash
pnpm spike-editor:build
SPIKE_REPORT=/tmp/t9.jsonl pnpm tauri dev --config \
  '{"build":{"beforeDevCommand":"pnpm spike-editor:preview","devUrl":"http://localhost:1420/?i=0"}}'
```

| Param | Effect |
| --- | --- |
| `i=0` | automatic plan: scale 1 three times, then 12 (~500 KB) and 48 (~2 MB) |
| `scale=N` | one run at scale N |
| `manual` | after the run, keeps the editor open and focused for the script below |

Latency is measured from the transaction to the next painted frame (`requestAnimationFrame` +
a message after it), one transaction per character, 200 characters in the middle of the
document. It does not include the time the OS takes to deliver the key event.

## Manual script (author, Wayland and pt-BR keyboard)

Open with `?scale=1&manual`, click in the text, and with ABNT2 and with US-intl:

1. ´ + a, e, i, o, u → á é í ó ú; ` + a → à; ^ + a, e, o → â ê ô; ~ + a, o, n → ã õ ñ; ¨ + u → ü
2. ç (ABNT2 key; US-intl: ' + c), Ç
3. Dead key followed by space (prints the accent itself) and by a consonant
4. ibus and fcitx5 (if installed): compose a word with the IME; the preedit shows underlined and
   commits without losing or duplicating characters
5. Paste a paragraph with accents; Ctrl+Z / Ctrl+Shift+Z; select with the mouse; Ctrl+F, replace
   "ação" by "acao" and back

Write down anything lost, duplicated or out of place, with the layout and the input method.
