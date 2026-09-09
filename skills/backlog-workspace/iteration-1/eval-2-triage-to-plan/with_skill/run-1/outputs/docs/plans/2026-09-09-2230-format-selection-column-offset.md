# Format Selection Column Offset Implementation Plan

> **For agentic workers:** Use executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Format Selection (and `editor.formatOnPaste`, which routes to the same provider) indent a top-level form that does not begin its line to the same columns whole-file cljfmt would.

**Tech Stack:** TypeScript, cljfmt-js (`reformatString`), Mocha through `@vscode/test-cli`.

**Origin:** `docs/backlog/format-selection-column-offset.md`, raised by a Codex review during `docs/plans/2026-09-03-2024-indent-on-paste.md`.

---

## Design

### The problem

`formatRange` in `src/fmt/cljfmtEngine.ts` (lines 208-265) reformats the
top-level forms that intersect the selected lines. It slices the document
from the first intersecting form's opener (after walking back over a glued
reader prefix such as `#_`, `'`, `#?@`) to the last intersecting form's
closer, hands that slice to `reformatString`, and replaces the slice with the
output. cljfmt sees the slice's first line starting at column 0, so it indents
the body as if the form began its line. When the form actually sits at
column N, every inner column comes out N short.

Verified against the current code with cljfmt-js 0.1.0 (cljfmt 0.16.5):

| text | selection | today | whole-file cljfmt |
| --- | --- | --- | --- |
| `^:private (a\nb)` | any line | `b` at 1 | `b` at 11 |
| `(x) (b\nc)` | line 1 only | `c` at 1 | `c` at 5 |
| `(x) #_(a\nb)` | line 1 only | `b` at 3 | `b` at 7 |
| `(x) (b\n     c)` (already correct) | line 1 only | `c` moved to 1 | unchanged |

The last row matters most: a range format over a correctly formatted mid-line
form *un-formats* it, so `editor.formatOnPaste` can damage clean code.

Selecting the opener's line as well hides the bug when a sibling form
precedes the opener, because that sibling then intersects the range and the
slice starts at its column-0 opener. `^:private (…)` is hit from any
selection, since metadata opens no span and the space stops the glued-prefix
walk. Glued prefixes on their own (`#_(a\nb)` at column 0) already come out
right and stay that way.

### The fix

Pad the slice with spaces up to the form's column before calling
`reformatString`, then strip the same padding from the output. This rests on
the property the existing "whitespace-only prefix" branch already relies on:
cljfmt never reindents the first line of its input (its indentation pass
rewrites whitespace after newlines), so a slice whose first line starts at the
form's real column produces exactly the columns the whole file would.

Concretely, after the existing `replaceStart` computation:

- `pad` is `" ".repeat(sliceStart - lineStart)` when the text before the form
  on its line is *not* whitespace-only, and `""` otherwise (that branch
  already replaces from `lineStart` and must keep moving a misindented
  top-level opener to column 0).
- Reformat `pad + slice`. If the output does not start with `pad`, return
  `null` (no edits). cljfmt does not do this today; the guard only exists so
  a future cljfmt that did could never produce an edit that eats real
  characters.
- Strip `pad` from the output. The existing "unchanged" check compares the
  stripped output with `slice`, so an already-correct mid-line form still
  yields `[]`.

Columns are UTF-16 units from the line start, the same measure `probeIndent`
uses (`win.start - openerLineStart`). A tab before the form counts as one
column, which is also how cljfmt counts it (`(x)\t(b\nc)` gives `c` at 5 both
ways). A newline inside a preceding multi-line string is a line start for
both sides too, so `"a\nb" (c\nd)` matches as well.

Prototyped on the compiled engine and compared with `reformatString` over
the whole text; every case in the tests below matched, including glued
prefixes behind a sibling (`(x) #_(a\nb)`, `(x) '(a\nb)`), space-separated
metadata behind a sibling, `#?@(:clj [a\nb])`, a comma before the form, a
selection spanning a mid-line form and a following column-0 form, a `let`
body, a nested `defn` body, an ns-aliased head with `nsContext`, and
`:indentation? false` (no edits).

### Decisions

- **Keep the glued-prefix backward walk; pad only the columns before it.**
  The prefix reaches cljfmt verbatim, as today. The entry's note stands:
  padding alone also matched, but deleting the walk buys nothing.
- **Leave the whitespace-only-prefix branch alone.** A misindented top-level
  form still moves to column 0, matching whole-file cljfmt when a form
  precedes it.
- **Pad to the form's *current* column, not the column cljfmt would give the
  line.** With `:remove-multiple-non-indenting-spaces? true`, whole-file
  cljfmt turns `(x)   (b\nc)` into `(x) (b\n     c)`; the range formatter
  keeps `(x)   (b` (outside the slice) and puts `c` at 7. Same for a
  misindented sibling before the form. The selected form is formatted
  relative to where it actually sits; text outside the selection is not
  touched. Accepted, and not covered by the whole-file comparison tests.
- **Return `null` when the padding does not survive**, rather than falling
  back to the unpadded output. `null` is the engine contract for "cannot
  format, do nothing"; the unpadded output is the bug this plan removes.
- **No README change.** The README scopes the byte-identical claim to Format
  Document, and Format Selection keeps other, intentional differences (the
  previous decision). Nothing documented the limitation, so nothing to
  retract.
- **Out of scope, noted for the record:** whole-file cljfmt leaves a
  misindented top-level opener alone when only a comment line (or nothing)
  precedes it, and the range formatter moves it to 0. CRLF documents come
  back with `\n` from both paths. Neither is caused or changed by this work.

### Testing

Unit tests live in `src/test/cljfmtEngine.test.ts` as a new
`cljfmtEngine.formatRange` suite. The oracle is `reformatString` over the
whole text with the same config: apply the range edits to the text and
assert string equality with the whole-file output. Because cljfmt never
reindents its first line, every text under comparison puts the form under
test on line 1 or later, with a *form* (not a comment) on the line before
when the case is about a whitespace-only prefix. Cases whose expected output
intentionally differs from whole-file cljfmt (the strict-config and
misindented-sibling cases above) pin an explicit expected string instead.

Integration coverage in `src/test/formatProvider.integration.test.ts`
extends the existing range-formatting tests with one Extension Host case
through `vscode.executeFormatRangeProvider`: selecting only the body line of
`(x) (b\nc)` gives `(x) (b\n     c)`. It pins the user-visible behavior
through the provider's line-range translation; the unit suite carries the
breadth.

## File Structure

| File | Change |
| --- | --- |
| `src/fmt/cljfmtEngine.ts` | Pad the range slice to the form's column and strip the padding from the output; update the `formatRange` comments. |
| `src/test/cljfmtEngine.test.ts` | New `cljfmtEngine.formatRange` suite comparing range edits with whole-file `reformatString`. |
| `src/test/formatProvider.integration.test.ts` | One mid-line Format Selection case through the provider. |
| `docs/backlog/format-selection-column-offset.md` | Status to done, with where it landed. |

## Implementation

Work on a branch from master. Use `/writing-clearly` for comments and prose.
`make test` wraps `xvfb-run -a npm test` on Linux and runs compilation and
lint through the npm `pretest` hook; run one suite with
`npm run compile-tests && xvfb-run -a npx vscode-test --grep "<suite name>"`.
The optional language-server end-to-end tests skip when `CLJ_PULSE_E2E_BIN`
is unset; report that as pending, not as failure.

### Task 1: Pad the range slice to the form's column

**Files:**
- Modify: `src/fmt/cljfmtEngine.ts`
- Test: `src/test/cljfmtEngine.test.ts`

- [ ] **Step 1: Write the failing unit tests**
  Add a `cljfmtEngine.formatRange` suite after the `selectWindow` suite.
  Import `reformatString`, `mergeConfig`, and `readNsContext` alongside the
  existing imports. Add two helpers local to the suite:

  - `applyEdits(text, edits)`: apply `SliceReplace` edits back to front
    (`text.slice(0, startOffset) + edit.text + text.slice(endOffset)`) and
    return the result; fail the test on `null`.
  - `rangeFormatted(text, startLine, endLine, lookup = DEFAULTS, nsContext?)`:
    `applyEdits(text, createCljfmtEngine(lookup, nsContext).formatRange(text, startLine, endLine))`.

  Then one `test` per group, each asserting
  `rangeFormatted(text, s, e)` equals `reformatString(text, lookup.config)`
  unless noted:

  - *mid-line form behind a sibling*: `"(x) (b\nc)"` lines 1-1 (the headline
    case); `"(a) (b) (c\nd)"` lines 1-1; `"(x)\t(b\nc)"` lines 1-1;
    `"(x),(b\nc)"` lines 1-1; `"\"s\" (b\nc)"` lines 1-1.
  - *space-separated metadata*: `"^:private (a\nb)"` lines 0-1 and lines 1-1.
  - *glued prefixes behind a sibling*: `"(x) #_(a\nb)"`, `"(x) '(a\nb)"`,
    `"(x) #?@(:clj [a\nb])"`, each lines 1-1. Also `"#?@(:clj [a\nb])"`
    lines 1-1 and `"#_(a\nb)"` lines 1-1 to pin that column-0 glued prefixes
    keep working.
  - *bodies deeper than one level*: `"(x) (let [y 1]\ny)"` lines 1-1;
    `"(x) (defn f [a]\n(let [b 1]\n(+ a b)))"` lines 1-2.
  - *selection spanning a mid-line form and a column-0 form*:
    `"(x) (b\nc)\n(d\ne)"` lines 1-3; `"(a\n b) (c\nd)\n(e\nf)"` lines 2-3.
  - *newline inside a preceding string is a line start*:
    `"\"a\nb\" (c\nd)"` lines 1-2.
  - *ns context still applies*: config
    `{:extra-indents {my.lib/mything [[:inner 0]]}}`, text
    `"(ns app (:require [my.lib :as ml]))\n(x) (ml/mything a\nb)"`, lines
    2-2, engine created with `readNsContext(text)`; expected whole-file
    output has `b` at column 6.
  - *already-correct mid-line form yields no edits*:
    `createCljfmtEngine(DEFAULTS).formatRange("(x) (b\n     c)", 1, 1)`
    deep-equals `[]`. Also `{:indentation? false}` over `"(x) (b\nc)"`
    lines 1-1 deep-equals `[]`.
  - *whitespace-only prefix still moves the opener to column 0*:
    `"(z)\n  (a\nb)"` lines 1-2 and lines 2-2 both equal the whole-file
    output `"(z)\n(a\n b)"`.
  - *pads to the form's current column* (explicit expected strings, since
    whole-file cljfmt also rewrites text outside the slice):
    `"  (x) (b\nc)"` lines 1-1 gives `"  (x) (b\n       c)"`; with config
    `{:remove-multiple-non-indenting-spaces? true}`, `"(x)   (b\nc)"` lines
    1-1 gives `"(x)   (b\n       c)"`.

- [ ] **Step 2: Run the suite to verify it fails**
  Run: `npm run compile-tests && xvfb-run -a npx vscode-test --grep "cljfmtEngine.formatRange"`
  Expected: FAIL on every mid-line, metadata, glued-behind-sibling, deeper
  body, spanning, string, ns-context, "already correct", and
  "current column" case (the body lands at the column-0 answer). The
  column-0 glued-prefix cases, the `:indentation? false` case, and the
  whitespace-only prefix cases PASS already.

- [ ] **Step 3: Implement the padding**
  In `formatRange`, after `replaceStart` is decided and `slice` is taken,
  compute `pad` as described in Design (empty when `replaceStart` was moved
  to `lineStart`, otherwise `sliceStart - lineStart` spaces), reformat
  `pad + slice`, return `null` when the output does not start with `pad`,
  strip it, and keep the existing unchanged check and edit shape against the
  stripped output. Replace the parenthetical in the glued-prefix comment
  ("Space-separated prefixes like `^:private (…)` are not recovered; Format
  Document handles those.") with a short comment on the padding: the slice's
  first line starts at the form's real column so cljfmt computes the columns
  the whole file would, and the padding is stripped from the output. Keep the
  existing comment about the first line never being reindented; the padding
  is a second use of the same property.

- [ ] **Step 4: Run the suite to verify it passes**
  Run: `npm run compile-tests && xvfb-run -a npx vscode-test --grep "cljfmtEngine"`
  Expected: PASS, including the existing `indentAt` and `selectWindow` suites.

- [ ] **Step 5: Commit**
  `git add src/fmt/cljfmtEngine.ts src/test/cljfmtEngine.test.ts`
  `git commit -m "Format Selection keeps a mid-line top-level form's column"`

### Task 2: Pin the behavior through the provider

**Files:**
- Test: `src/test/formatProvider.integration.test.ts`

- [ ] **Step 1: Add the integration case**
  Next to "reader prefixes stay part of the range-formatted form", add a test
  "a top-level form that does not begin its line keeps its column": open
  `"(x) (b\nc)"`, format `new vscode.Range(1, 0, 1, 2)`, and assert the text
  becomes `"(x) (b\n     c)"`. Keep the existing comment style: expected
  strings come from JVM cljfmt 0.16.5 on the same input.

- [ ] **Step 2: Run the integration suite**
  Run: `npm run compile-tests && xvfb-run -a npx vscode-test --grep "Format Document / Selection"`
  Expected: PASS (the fix from Task 1 is already in place; this test guards
  the provider wiring and the line-range translation in `src/extension.ts`).

- [ ] **Step 3: Run the full check**
  Run: `make check`, then `git diff --check`.
  Expected: lint, compile, and the whole suite exit 0; the optional
  language-server tests are pending when `CLJ_PULSE_E2E_BIN` is unset.

- [ ] **Step 4: Commit**
  `git add src/test/formatProvider.integration.test.ts`
  `git commit -m "Cover mid-line Format Selection through the provider"`

### Task 3: Close the backlog entry

**Files:**
- Modify: `docs/backlog/format-selection-column-offset.md`
- Modify: `docs/plans/2026-09-09-2230-format-selection-column-offset.md`

- [ ] **Step 1: Mark the entry done**
  Change `**Status: planned**` to `**Status: done**` and replace the `Plan:`
  line under it with
  `Landed in docs/plans/2026-09-09-2230-format-selection-column-offset.md (<commit or PR>).`
  using the Task 1 commit hash, or the PR number once one exists.

- [ ] **Step 2: Record results in this plan**
  Tick the completed steps, add verification notes under each task (test
  counts, pending tests, deviations), and fill in a short **Completed**
  section at the end: what changed, verification, issues, deviations.

- [ ] **Step 3: Commit**
  `git add docs/backlog/format-selection-column-offset.md docs/plans/2026-09-09-2230-format-selection-column-offset.md`
  `git commit -m "Close the format-selection column offset backlog entry"`
