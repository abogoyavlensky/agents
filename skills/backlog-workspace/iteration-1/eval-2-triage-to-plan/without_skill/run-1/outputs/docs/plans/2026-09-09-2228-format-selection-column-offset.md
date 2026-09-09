# Format Selection Column Offset Implementation Plan

> **For agentic workers:** Use executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Format Selection (and `editor.formatOnPaste`) indent a top-level form that does not start its line at the columns whole-file cljfmt would produce.

**Tech Stack:** TypeScript VS Code extension, bundled `@abogoyavlensky/cljfmt-js`, Mocha unit and integration tests via `@vscode/test-cli`.

Picks up `docs/backlog/format-selection-column-offset.md`.

---

## Design

### Problem

`formatRange` in `src/fmt/cljfmtEngine.ts` slices the document from the
opener of the first intersecting top-level form and hands that slice to
`reformatString`. cljfmt indents the slice as if its first character sat at
column 0. When the form does not begin its line, every inner column comes out
short by the form's real column:

```clojure
^:private (a
b)        ; Format Selection puts `b` at column 1; whole-file cljfmt puts it at 11

(x) (b
c)        ; Format Selection puts `c` at column 1; whole-file cljfmt puts it at 5
```

Reader prefixes glued to the opener (`#_`, `'`, `#?`, `@`, `#(`) are already
handled: the backward walk in `formatRange` extends the slice over them, so
cljfmt sees them and indents accordingly. The bug is only about what sits
*before* the slice on the same line.

### Fix

Prepend spaces to the slice so its first character lands at the form's real
column, reformat, then strip the same number of spaces from the output.

This leans on the property `formatRange` already relies on: cljfmt never
reindents the first line of its input (its indent pass rewrites whitespace
after newlines only). A slice that starts at the form's real column therefore
gets the same inner columns the whole file would, and the padding comes back
verbatim at the front of the output.

Verified against the bundled cljfmt-js (default config) before writing this
plan — every padded result equals `reformatString(wholeText).slice(formStart)`:

| input | today | padded (= whole-file) |
| --- | --- | --- |
| `^:private (a\nb)` | `b` at 1 | `b` at 11 |
| `(x) (b\nc)` | `c` at 1 | `c` at 5 |
| `(x) #_(a\nb)` (padding plus glued prefix) | `b` at 3 | `b` at 7 |
| `(x) '(a\nb)` | `b` at 2 | `b` at 6 |
| `(x) (b\n(c\nd))` (nested body lines) | `(c` at 1, `d` at 2 | `(c` at 5, `d` at 6 |
| `(x) (let [y 1]\ny)` (block rule) | `y` at 2 | `y` at 6 |
| `(x)\t(b\nc)` (tab before the form) | `c` at 1 | `c` at 5 |
| `(x) (b\nc)\n(y\nz)` (second form at column 0 in the same slice) | `c` at 1, `z` at 1 | `c` at 5, `z` at 1 |
| `#_(a\nb)`, `#(a\nb)`, `#?@(:clj [a\nb])` (nothing before the form) | already right | unchanged |

The tab row shows cljfmt counts a tab as one column, so padding by the number
of UTF-16 units before the form matches cljfmt without any tab expansion.

### Where the padding goes

`formatRange` today computes:

- `formStart` — the opener, walked back over any glued reader prefix.
- `lineStart` — start of the opener's line.
- `sliceStart = formStart`; `replaceStart` moves to `lineStart` when the text
  between them is whitespace only.

The change fits between the existing `replaceStart` decision and the
`reformatString` call:

- **Whitespace-only prefix (existing branch, unchanged).** No padding. The
  slice starts at the form, so a misindented top-level form moves to column 0
  — exactly what whole-file cljfmt does.
- **Anything else on the line before the form (new).** `pad` is
  `sliceStart - lineStart` spaces. Reformat `pad + slice`. If the output does
  not start with `pad`, return `null` (the "cannot format — do nothing"
  contract; never emit a mis-shaped edit). Otherwise strip `pad` and treat the
  remainder as `out`.
- The no-op check stays `out === slice && replaceStart === sliceStart`, now
  against the stripped output, so an already-correct mid-line form still
  yields `[]`.
- The glued-prefix backward walk stays as is. Padding is measured from
  `sliceStart` (after the walk), so the prefix reaches cljfmt verbatim and the
  padding covers only the columns before it. Padding alone happened to match
  every measured case, but that is too little evidence to delete the walk.

Roughly six lines. No new functions, no new files, no config.

### Comment update

The comment above the backward walk currently ends with "(Space-separated
prefixes like `^:private (…)` are not recovered; Format Document handles
those.)" That sentence becomes false. Replace it with one sentence saying the
padding below restores the form's column for whatever precedes the slice on
its line. Keep the explanation of *why* the first line is safe to pad (cljfmt
never reindents its first line) next to the padding code — it is the
property the whole approach depends on.

### Testing

**Unit** (`src/test/cljfmtEngine.test.ts`, new suite
`cljfmtEngine.formatRange`). The engine is pure, so the tests compare
`formatRange` output with `reformatString` over the whole text — the oracle
the backlog entry used. A small local helper applies the returned slice edit
to the input text; the cases above are the table. Two cases assert exact
strings instead of the oracle:

- whitespace-only prefix still moves the form to column 0 while leaving the
  untouched sibling alone (`(a\nb)\n  (c\nd)`, lines 2–3 →
  `(a\nb)\n(c\n d)`); the oracle would also reindent `(a\nb)`;
- an already-formatted mid-line form yields `[]`.

Avoid `def`-prefixed fake head symbols — cljfmt's default `#"^def"` rule would
make column assertions pass for the wrong reason.

**Integration** (`src/test/formatProvider.integration.test.ts`). One test
through `vscode.executeFormatRangeProvider`, so the real provider wiring is
exercised: `(x) (b\nc)` with the selection on line 1 →
`(x) (b\n     c)`, and `^:private (a\nb)` with the selection on line 1 →
`^:private (a\n           b)`.

**Not covered by change:** the structural engine (`formatRange` there is
line-based and never had this problem), Format Document, and Enter.

### Documentation

README's Formatting section never advertised the limitation, so no README
change. Close the backlog entry.

## File Structure

- Modify: `src/fmt/cljfmtEngine.ts` — pad the range slice and strip the
  padding; update the backward-walk comment.
- Modify: `src/test/cljfmtEngine.test.ts` — new `cljfmtEngine.formatRange`
  suite comparing against whole-file `reformatString`.
- Modify: `src/test/formatProvider.integration.test.ts` — one Format
  Selection test for a mid-line form.
- Modify: `docs/backlog/format-selection-column-offset.md` — mark done.

Work on a branch `format-selection-column-offset` from `master`, as the
previous fixes did (`comment-sign-setting`, `fix-lsp-navigation-in-jar`).

Test commands below assume a display; on Linux prefix them with
`xvfb-run -a` (see `Makefile`).

---

### Task 1: Pad the range slice to the form's column

**Files:**
- Modify: `src/fmt/cljfmtEngine.ts`
- Test: `src/test/cljfmtEngine.test.ts`

- [ ] **Step 1: Write the failing unit tests**
  Add a `suite("cljfmtEngine.formatRange", …)` after the `indentAt` suite.
  Add a local helper that runs `createCljfmtEngine(DEFAULTS).formatRange(text, startLine, endLine)`,
  asserts it returned exactly one `slice` edit, and returns the text with
  that edit applied. Import `reformatString` from `@abogoyavlensky/cljfmt-js`
  for the oracle.

  Tests (one `test` per row, or one table-driven test with each input in the
  assertion message):
  - `^:private (a\nb)`, lines 1–1 → equals `reformatString(text, defaultConfig)`.
  - `(x) (b\nc)`, lines 1–1 → equals the oracle.
  - `(x) #_(a\nb)`, lines 1–1 → equals the oracle (padding plus glued prefix).
  - `(x) '(a\nb)`, lines 1–1 → equals the oracle.
  - `(x) (b\n(c\nd))`, lines 1–2 → equals the oracle.
  - `(x) (let [y 1]\ny)`, lines 1–1 → equals the oracle.
  - `(x)\t(b\nc)`, lines 1–1 → equals the oracle (tab counts as one column).
  - `(x) (b\nc)\n(y\nz)`, lines 1–3 → equals the oracle (padding touches
    only the first line; the second form stays at column 0).
  - `#?@(:clj [a\nb])`, lines 0–1 → equals the oracle (reader-conditional
    splicing, never exercised before).
  - `(a\nb)\n  (c\nd)`, lines 2–3 → exact string `(a\nb)\n(c\n d)` (the
    whitespace-only branch is untouched).
  - `(x) (b\n     c)`, lines 1–1 → `formatRange` returns `[]`.

- [ ] **Step 2: Run the suite to verify it fails**
  Run: `npm run compile-tests && npx vscode-test --label unit -g "cljfmtEngine.formatRange"`
  Expected: the oracle comparisons for mid-line forms FAIL with inner lines
  short by the form's column (for example `"(x) (b\n c)"` vs
  `"(x) (b\n     c)"`); the `#?@`, whitespace-only, and `[]` cases pass
  already.

- [ ] **Step 3: Implement the padding**
  In `formatRange`, after `replaceStart` is decided and before
  `reformatString`: when `replaceStart === sliceStart` (something other than
  whitespace precedes the form on its line) and `sliceStart > lineStart`,
  build `pad` of `sliceStart - lineStart` spaces, reformat `pad + slice`,
  return `null` if the output does not start with `pad`, else strip it.
  When `replaceStart !== sliceStart`, reformat `slice` unchanged. Keep the
  existing no-op check against the stripped output.

  Rewrite the tail of the backward-walk comment (the parenthetical about
  `^:private`) to say the padding below restores the form's column; keep the
  "cljfmt never reindents the first line" explanation beside the padding.

- [ ] **Step 4: Run the unit tests**
  Run: `npm run compile-tests && npx vscode-test --label unit -g "cljfmtEngine"`
  Expected: PASS, including the existing `indentAt` and `selectWindow` suites.

- [ ] **Step 5: Commit**
  `git commit -am "Pad range-format slices to the form's column"`

### Task 2: Cover Format Selection end to end

**Files:**
- Test: `src/test/formatProvider.integration.test.ts`

- [ ] **Step 1: Add the integration test**
  In the `Format Document / Selection (integration)` suite, next to
  "reader prefixes stay part of the range-formatted form", add
  "a form that does not start its line keeps its column": open
  `(x) (b\nc)`, format `new vscode.Range(1, 0, 1, 2)`, expect
  `(x) (b\n     c)`; then open `^:private (a\nb)`, format the same range,
  expect `^:private (a\n           b)`. Both expected strings match whole-file
  cljfmt (see the Design table).

- [ ] **Step 2: Run the integration suite**
  Run: `npm run compile-tests && npm run compile && npx vscode-test --label unit -g "Format Document / Selection"`
  Expected: PASS.

- [ ] **Step 3: Run the full check**
  Run: `make check`
  Expected: lint, compile, and the whole test suite pass.

- [ ] **Step 4: Commit**
  `git commit -am "Test Format Selection on a mid-line top-level form"`

### Task 3: Close the backlog entry

**Files:**
- Modify: `docs/backlog/format-selection-column-offset.md`

- [ ] **Step 1: Mark it done**
  Change `**Status: open**` to `**Status: done**` and add a line beneath it
  saying the fix landed on branch `format-selection-column-offset` via this
  plan (`docs/plans/2026-09-09-2228-format-selection-column-offset.md`).
  Leave the rest of the entry as the record of the analysis.

- [ ] **Step 2: Commit**
  `git commit -am "Close backlog: format-selection column offset"`
