# Backlog entry template

Copy the skeleton, replace every bracketed part, delete the section you do not
need (`Fix` or `Why it is left alone` — keep at least one). Headings are
`##`; the status line is the only line whose exact text matters.

```markdown
# [A sentence stating the observed behavior, e.g. "Format Selection indents a mid-line top-level form from column 0"]

**Status: open**

## Problem

[What happens. How to trigger it — a minimal repro if the issue is about input.]

[The cause, with file paths and line numbers: `src/fmt/cljfmtEngine.ts`,
`syntaxes/clojure.tmLanguage.json:303`. Quote the offending line or pattern if
it is short.]

[How narrow it is in practice: what a user must do to hit it. Which related
cases already work, so nobody re-fixes them.]

## Fix          <- use when the shape of the solution is known

[The mechanism and the invariant it relies on. Measurements or a small table
if you have them.]

Notes for whoever picks this up:

- [What to keep, what not to touch, what to cover in tests.]

[Size estimate: "about six lines in `formatRange` plus unit cases in ...".]

## Why it is left alone     <- use when the decision is to not fix it now

[What a naive fix would break. What a real fix would require. What would have
to change for it to be worth doing.]

## Origin

[Where it surfaced and when: the plan, review, or task, by path, with the
date. If a review misreported it, what the review said versus what is true.]
```

Filename: the title as a kebab-case slug, shortened to the distinctive part —
`format-selection-column-offset.md`, `palette-activation-gate.md`. No date,
no number.

Status transitions, each on the line after the title:

```markdown
**Status: open**

**Status: planned**
Plan: docs/plans/2026-09-10-1412-format-selection-column-offset.md

**Status: done**
Landed in docs/plans/2026-09-10-1412-format-selection-column-offset.md (#31).
```
