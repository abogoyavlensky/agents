# Report and presentation

Read this once verification is done. Two outputs come out of a round: `report.md` in the round folder,
which is the full record, and the turn you write in chat, which is a decision brief. The record already
holds everything, so the brief does not need to repeat it. A reader scrolling past detail to find the
decision is worse off, not better.

## What gates

A finding **gates** (blocks "done") when its severity after verify is `critical` or `major` and its
verdict is:

- `confirmed` or `refined`, or
- `unverified`, because its verifier failed. A failed check is not a clean result, so an unverified
  serious finding is treated as real until a later round settles it. Label it `unverified`.

Minors, open questions, pre-existing, immaterial and rejected findings never gate.

## `report.md` template

```markdown
# panel-review: <task> / <round>

- scope: <one line, e.g. "branch improve-review vs master, 14 files, +820 −410">
- mode: focused | full | plan
- base: <the base recorded in round 1's scope.md>
- finders: expected C, X · reported C, X · missing reviewer: none | X (<reason>)
- verifiers: 1 of 1 returned · missing verifier: none | <n> (<reason>)
- counts: <n> gating · <n> minor · <n> open questions · <n> pre-existing · <n> immaterial · <n> rejected

## Gating

### M1: <title> [major, confirmed, conf 95, sources C2+X1]
- location: <path:line>
- trigger: <…>
- fix: <…>

<body, refined values applied>

## Minor

### M4: <title> [minor, confirmed, conf 60, sources C5]
- location / trigger / fix as above

<body>

## Open questions
- C3?: <question> (<location>)

## Pre-existing
- X4: <title> (<location>), <severity>

## Immaterial
- M6: <title>, <the verifier's reason in a clause>

## Rejected
- M7: <title>, <the verifier's reason in a clause>

## Verdict
<two or three sentences: clean or not, what it turns on, what happens next>
```

Leave out any section that would be empty, except `## Verdict`. When several sources agree, show it in
the heading (`sources C2+X1`). That is the strongest signal the panel gives.

## When the user asked for the review directly

Write the brief in this order:

1. **Header**: one or two sentences on what was reviewed and what it sets out to do.
2. **Completeness**: finders expected vs reported. Lead with any missing reviewer or verifier.
3. **Counts**: one line.
4. **Gating, then minor findings**, each as in the report: heading, location, trigger, fix and body.
   The user acts on each of these, so each one gets its full body.
5. **Open questions, pre-existing, immaterial, rejected**: one line each.
6. **Verdict**: two or three sentences, then the report path.

End the turn with `AskUserQuestion`, and put nothing after it. The question text opens with the
verdict in one sentence. The recommended option goes first, with its reason in its description. Options
name the findings they act on by title, not by count, because "fix the two majors" cannot be checked
at a glance and "fix the backoff reset and the missing auth check" can. Typical options:

- Fix the gating findings (<titles>), then re-review (recommended when anything gates)
- Fix everything, including the minors
- Fix selected findings (the user names them)
- Leave it as is

Fix only what the user picks. After fixing, offer the next round. Do not start it on your own.

## When another skill called it

When executing-plans, makegoal or fastplan runs the review, nobody is there to answer a question, and
the caller decides what to fix. Do not ask. Return this block and stop:

```markdown
panel-review <task>/<round> (<mode>): <clean | N gating | error: all finders failed>
- Gating: M1 <title> (path:line); M2 <title> (path:line, unverified)
- Minor: M4 <title> (path:line)
- Missing reviewer: none | X (<reason>)
- Missing verifier: none | <n> (<reason>)
- Report: .tmp/review/<task>/<round>/report.md
```

How a caller reads it:

- `clean` passes. A missing reviewer makes it a weaker review, but still a valid one.
- `N gating` does not pass. Fix the gating findings and run the next round, or stop when the round
  rules say to.
- `error` means no review happened. It never passes. Report the reason to the user.
- A missing verifier means its findings are `unverified`. An unverified critical or major already
  counts in `N gating`, so the result can never read `clean` because of a verifier that did not run.
