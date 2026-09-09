Plan written and committed: `docs/plans/2026-09-09-2228-format-selection-column-offset.md` on a new branch `format-selection-column-offset` (commit `5e66990`, branched from `master` at `53a299d`). No implementation started; the backlog entry stays open until the fix ships.

**What the plan does.** In `formatRange` (`src/fmt/cljfmtEngine.ts`), when something other than whitespace precedes the top-level form on its line, prepend `sliceStart - lineStart` spaces to the slice before `reformatString`, then strip the same padding from the output. cljfmt never reindents the first line of its input, so a slice that starts at the form's real column gets the columns the whole file would. About six lines, no new functions or files.

**Calls I made without you:**

- **Verified the fix against the bundled cljfmt-js before planning**, not just the four rows in the backlog. All thirteen cases matched whole-file `reformatString`, including the ones the backlog left open: padding plus a glued prefix (`(x) #_(a\nb)`), nested body lines, a `let` block rule, a tab before the form (cljfmt counts a tab as one column, so no tab expansion is needed), and a second form at column 0 inside the same slice. The plan's expected columns come from those runs.
- **Keep the glued-prefix backward walk** and pad only the columns before it, as the backlog suggested. Padding alone matched every case, but that is not enough evidence to delete the walk.
- **Whitespace-only prefix branch is untouched**: a misindented top-level form still moves to column 0.
- **If cljfmt ever fails to return the padding verbatim, `formatRange` returns `null`** (the existing "cannot format, do nothing" contract) rather than emitting a mis-shaped edit. It never happened in testing; this is a guard, not a code path we expect to hit.
- **Tests**: a new `cljfmtEngine.formatRange` unit suite that compares `formatRange` output with whole-file `reformatString` (the oracle you used in the backlog), plus one integration test through `vscode.executeFormatRangeProvider`. Reader-conditional splicing (`#?@`) gets its first test, per the backlog note.
- **No README change**: the Formatting section never advertised the limitation.
- The plan's last task closes the backlog entry (`Status: done` with a pointer to the plan and branch).

**Skipped:** the fastplan Codex review pass; it was not run in this session. executing-plans will still give the plan its own critical read before running it.

Three tasks: pad and unit-test, integration test plus `make check`, close the backlog entry. When you are back, say the word and I will run it with executing-plans on this branch.
