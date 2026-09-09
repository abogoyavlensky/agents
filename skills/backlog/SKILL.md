---
name: backlog
description: Work with a project's file-based backlog in docs/backlog/ — add an entry for a known issue, limitation, or idea that is deliberately not being fixed now; list what is open; close entries when work ships; and triage an entry into an implementation plan in docs/plans/ when it is picked up. Use this whenever the user says "backlog", "add to the backlog", "note this for later", "park this", "defer this", "not now", "known issue", "known limitation", "what's in the backlog", "triage", "pick something up from the backlog", or mentions docs/backlog/ — and also, without being asked, when you discover a real bug or gap that is out of scope for the current task, when a plan or code review defers something, or when a plan you are executing has a "backlog the X" task. If the repo has no docs/backlog/ yet, this skill bootstraps it.
---

# Backlog

A backlog entry is a letter to whoever picks the issue up months from now, with no chat history and no memory of today. It has one job: let that person decide in five minutes whether to act, and start acting without re-doing the investigation. Everything below follows from that.

The convention lives in the repo under `docs/backlog/`, next to `docs/plans/` where implementation plans go. A backlog entry is what a problem looks like *before* anyone commits to fixing it; a plan is what it looks like *after*. Triage is the move from one to the other.

## The rules

These are the project's rules (mirrored in `AGENTS.md`); keep them in sync if you change either side.

- **One file per issue**, in `docs/backlog/`, named after the issue in kebab-case: `format-selection-column-offset.md`. No date, no number — the name is the identity, and dates belong to plans.
- **Every file starts with a title line and a status line:** `# <title>` then `**Status: open**`. Status is the one machine-readable field; everything relies on it.
- **Statuses:** `open` (nobody is on it), `planned` (a plan exists in `docs/plans/`, see the line after the status), `done` (shipped; the line after the status says where it landed). Anything not `done` counts as open when listing.
- **Files are never deleted.** A done entry is the record of why the problem existed and where it went. Closing means changing the status, not removing the file.
- **A backlog entry is its own commit**, never mixed with code. `Backlog: <what>` or `docs: backlog <what>` as the message. That way the entry can be read, reverted, or cherry-picked on its own.
- **Not a TODO list.** If you can fix it now inside the current task's scope, fix it. The backlog is for things a decision was made *not* to do now: too narrow, needs a bigger change, wrong layer, or simply out of scope for the plan in flight.

## Adding an entry

Reach for this when you have a concrete problem that will not be solved now. Typical moments: a plan explicitly defers part of the scope; a code review (yours or Codex's) surfaces a real issue outside the task; you notice something while reading code for something else; the user says "note this for later".

Before writing, make sure you actually understand the problem — a vague entry is worse than none, because it looks like knowledge and isn't. Reproduce or at least locate the cause in code. If a review reported it, verify it yourself: reviews get the symptom right and the cause wrong often enough that the entry should say which part is real.

Then write the file following `references/entry-template.md`. The shape that has worked:

1. **Title** — a sentence stating the observed behavior, not a category. "Format Selection indents a mid-line top-level form from column 0", not "Format selection bug". Someone scanning filenames and titles should know what is wrong without opening the file.
2. **`**Status: open**`** on its own line right after the title.
3. **`## Problem`** — what happens, how to trigger it, and the cause with file paths and line numbers (`src/fmt/cljfmtEngine.ts`, `syntaxes/clojure.tmLanguage.json:303`). Include a minimal repro when the issue is about input (a code snippet, a manifest entry). Say how narrow it is in practice: what a user has to do to hit it. Mention what related cases already work, so the reader does not re-fix them.
4. **One of:**
   - **`## Fix`** or **`## Proposed fix`** — when you know the shape of the solution. Give the mechanism, the invariant it relies on, any measurements that back it up, and "notes for whoever picks this up": what to keep, what not to touch, what to test. An estimate of size ("about six lines plus unit cases") helps triage more than it costs.
   - **`## Why it is left alone`** — when the decision is to *not* fix it, or not with the tools at hand. Explain the tradeoff honestly: what a naive fix would break, what a real fix would require, what would have to change for it to be worth doing. This section is the one most often skipped and the one that saves the most time later, because it stops the next person from re-discovering the dead end.
5. **`## Origin`** — where it came from and when: the plan or review that surfaced it (`docs/plans/2026-09-05-1445-minimize-command-palette.md`), the date, and if a review misreported it, what the review said versus what is true.

Write in plain prose, past and present tense, no hedging. Cite files by path; cite line numbers when they help someone jump there, knowing they drift. Link related plans and other backlog entries by path. Read the two examples in `references/examples.md` — one with a known fix, one deliberately left alone — before writing your first entry in a repo; matching their register matters more than matching their headings.

Then commit the file on its own:

```bash
git add docs/backlog/<slug>.md && git commit -m "Backlog: <what the entry is about>"
```

When the entry is created as a task inside a plan being executed, the plan already has a "Backlog the X" task with its own commit step; follow it.

## Listing

"What's in the backlog?" means every entry whose status is not `done`, with its title. Get it deterministically rather than from memory:

```bash
for f in docs/backlog/*.md; do
  s=$(grep -m1 -o 'Status: [a-z]*' "$f" | cut -d' ' -f2)
  [ "$s" != done ] && printf '%s\t%s\t%s\n' "$s" "$f" "$(head -1 "$f" | sed 's/^# //')"
done
```

Present each entry as its title plus, if useful, a one-line summary and the size hint from its fix section. If the user is choosing what to work on next, add a recommendation based on value versus size — that is what the "Fix" and "Why it is left alone" sections were written for. Listing is a read of the files, not an audit: the staleness check against the code happens at triage, for the one entry being picked up. Do not pad the list with done entries unless asked for history.

## Triage: backlog entry to plan

Triage is when the user decides an entry is worth doing now. The entry becomes the input to a plan, and the plan takes over as the working document. The steps:

1. **Re-read the entry against the current code.** Entries age: line numbers drift, a refactor may have removed the cause, or another change may have fixed it by accident. Check every file reference and the repro. If the problem is already gone, skip planning: mark the entry done with a line saying which commit or plan fixed it, commit, and tell the user.
2. **Plan it.** Use the `fastplan` skill if it is available — it explores, decides, presents the design once, and writes `docs/plans/YYYY-MM-DD-HHMM-<topic>.md`. Without it, write the plan yourself in that location with `date +%Y-%m-%d-%H%M` for the prefix and the header/task shape the repo's other plans use. Either way the backlog entry does most of the design work: fold its Problem into the plan's Design section, carry over the Fix or Proposed fix as the approach (or explain in the plan why you diverged from it), keep its "notes for whoever picks this up" as constraints, and cite the entry by path so the plan's origin is traceable.
3. **Make closing the entry part of the plan.** Add a final task to the plan: "Close the backlog entry" — change `**Status: open**` to `**Status: done**`, add a line under it saying where it landed (the plan path and the PR or commit), commit. Putting it in the plan means executing-plans does it and nobody has to remember.
4. **Mark the entry planned.** Once the plan file exists, change the entry's status line to `**Status: planned**` and add a line right under it: `Plan: docs/plans/<file>.md`. Commit this together with the plan. The entry still shows in listings (it is not done), but anyone reading it can see work is in flight and where.
5. **Hand off** to executing-plans as fastplan normally does.

If the user wants to triage the whole backlog rather than one entry, list it first, recommend an order, and take entries one at a time — one plan per entry unless two are genuinely the same change.

## Closing an entry outside a plan

Sometimes an entry gets fixed incidentally: a refactor removes the cause, or the fix was tiny and done in passing. When you notice, close it: `**Status: done**`, then a line stating where it landed (`Landed in <commit or PR>` or `Fixed by docs/plans/<file>.md`), committed on its own or with the fixing change. Do not leave entries that describe a problem that no longer exists — a stale open entry costs the next reader the same investigation it was meant to save.

## Bootstrapping a repo that has no backlog

If `docs/backlog/` does not exist and the user wants to record something for later, set it up in the same commit as the first entry:

1. `mkdir -p docs/backlog`.
2. Add the section from `references/agents-md-section.md` to the repo's `AGENTS.md` (or `CLAUDE.md`, whichever the repo uses as the agent notes file; create `AGENTS.md` if there is none). This is what tells future sessions the convention exists, so it is not optional.
3. Write the first entry as above.

Do not bootstrap on a whim — a backlog directory in a repo where nobody triages it is noise. Do it when there is a real entry to write.

## Things that go wrong

- **Writing the entry from memory of the symptom instead of from the code.** The Problem section should have paths in it. If it has none, you have not looked.
- **Filing what should be fixed now.** If the fix is a few lines inside the file you are already editing and inside the task's scope, just do it and mention it in the commit.
- **Deleting or renaming entries to "clean up".** The name is the identity other files link to; the file is the history.
- **Bundling the entry into a feature commit.** Then it cannot be found by `git log -- docs/backlog`.
- **Forgetting to close.** Every plan born from an entry ends with closing it; every incidental fix that touches an entry's cause updates its status.
- **Changing the rules here without changing `AGENTS.md`** (or the reverse). The repo's `AGENTS.md` is what a session without this skill reads; the two must agree.
