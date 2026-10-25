# Output

Return **only** the findings, in the exact template below, most severe first. No preamble, no summary,
no closing remarks. Another model parses this output by its headings and field names, and a finding
outside the template is lost.

Use the id prefix given in the round context (for example `C`), numbered from 1.

## Findings

```markdown
### C1: <the mechanism you established, in one line>
- location: <path>:<line> | <path>:<line>-<end_line> | <path> | —
- severity: critical | major | minor
- confidence: <0-100>
- lenses: <lens>, <lens>
- trigger: <the input, state or sequence that causes it, in one sentence>
- fix: <the smallest change that resolves it, or —>

<What goes wrong and what it causes. Mark which part you observed in the code and which part you
infer from it.>
```

Example:

```markdown
### C2: Backoff delay is never reset after a successful attempt
- location: app/sync/worker.go:88
- severity: major
- confidence: 85
- lenses: bugs
- trigger: two 503 responses, then a success, then another 503
- fix: set `delay = baseDelay` after a successful attempt at worker.go:95

Observed: `delay` doubles at :88 on each failure and nothing writes it back on success (:95 returns
early). Inferred: after any transient outage every later retry waits the maxed-out delay, so the worker
stalls for minutes on a single blip.
```

## Pre-existing problems

A real defect whose failure was already reachable before the change: same template, plus the line
`- pre_existing: yes`. A defect in untouched code that the change makes reachable belongs to the
change, so do not mark it pre-existing.

## Open questions

Something you could not settle from the code and the context, where the answer changes whether there
is a defect. Use `?` after the id and only a location and a body:

```markdown
### C3?: Is `Config.Timeout` meant to be seconds or milliseconds?
- location: app/config.go:41

The new caller at client.go:20 passes `5000`; every other caller passes values under 60.
```

## Nothing found

If you have no findings and no questions, return exactly:

```
No findings.
```
