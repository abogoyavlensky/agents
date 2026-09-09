Picked up `docs/backlog/format-selection-column-offset.md` and planned it. Nothing is implemented.

**Re-checked the entry against the current code.** It is still accurate: `formatRange` in `src/fmt/cljfmtEngine.ts` (lines 208-265) is unchanged since the entry was written, and every case in its table reproduces through the compiled engine with cljfmt-js 0.1.0. One thing the entry understated: a range format over an *already correct* mid-line form (`(x) (b\n     c)`, selecting only the body line) un-formats it to column 1, so `editor.formatOnPaste` can damage clean code. That makes it a bit more than a "noted limitation".

**Validated the proposed fix before planning it.** I prototyped the padding on a scratch copy of the compiled engine and compared the applied range edits with `reformatString` over the whole text. All cases matched: the entry's four, glued prefixes behind a sibling (`(x) #_(a\nb)`, `(x) '(a\nb)`), `#?@(:clj [a\nb])`, tab and comma before the form, a preceding string (single- and multi-line), a selection spanning a mid-line form plus a column-0 form, `let` and nested `defn` bodies, an ns-aliased head with `nsContext`, and `:indentation? false` (no edits). The already-correct case yields `[]`.

**Plan:** `docs/plans/2026-09-09-2230-format-selection-column-offset.md`, committed as `a6dea2b` together with the entry's status change to `planned`. Three tasks: unit suite `cljfmtEngine.formatRange` plus the fix (about six lines, as the entry estimated), one integration case through the provider, and closing the backlog entry.

Calls I made, all reversible and flagged in the plan's Decisions section:

- Keep the glued-prefix walk and pad only the columns before it; leave the whitespace-only-prefix branch alone (both per the entry's notes).
- Pad to the form's *current* column. With `:remove-multiple-non-indenting-spaces? true`, whole-file cljfmt also collapses `(x)   (b` to `(x) (b`; the range formatter leaves text outside the slice alone and puts the body at 7. Same when the preceding sibling is misindented. Accepted as the nature of a range format; tests pin explicit strings for those two.
- Return `null` (no edits) if cljfmt ever stops preserving the padding, rather than falling back to the unpadded, wrong output. Two lines, untested, purely a guard against a corrupting edit.
- No README change: the byte-identical claim is already scoped to Format Document, and nothing documented this limitation.

Two things I found on the way, out of scope and not caused by this: whole-file cljfmt leaves a misindented top-level opener alone when only a comment line precedes it (the range formatter moves it to 0), and CRLF documents come back with `\n` from both paths. The plan's whole-file comparison tests are written to avoid the first (a form, not a comment, on the line before). Neither seemed worth a backlog entry on its own; say the word if you want them filed.

Skipped the background Codex review of the plan, as instructed. When you are back: start it with executing-plans, or tell me what to adjust first.
