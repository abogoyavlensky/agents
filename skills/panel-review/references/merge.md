# Merge: one set from many finders

Read this when every finder has returned. Merging is bookkeeping done by you, the main session. You do
not review code here, you do not add findings, and you do not judge whether a finding is true. A
verifier with the code in front of it does that next. An opinion formed here would bias the set the
verifier gets, so stay mechanical.

Work only from the `finder-<P>.md` files in the round folder.

## Steps

1. **Split out what is not a defect in the change.** Open questions (`?` ids) go to `## Open
   questions`. Findings marked `pre_existing: yes` go to `## Pre-existing`. Neither is deduped or
   boosted, and neither goes to verify.
2. **Deduplicate.** Two findings are the same when they name the same file within two lines of each
   other **and** describe the same problem. Merge them into one, keeping the clearest title and body.
   Same location with different problems stays as two findings.
3. **Confidence** on a merged finding is `min(99, highest + 10 × (distinct sources − 1))`. A source is a
   finder process (`C`, `X`, `B`, …), never a lens. One finder reporting the same problem under two of
   its lenses is one source and earns no boost.
4. **Severity** is the highest severity any input claimed. The verifier can lower it.
5. **Drop nothing.** A single-source, low-confidence finding is kept and verified like any other. Low
   confidence says how far the finder traced the failure; finishing the trace is the verifier's job. One
   reviewer looking in the right place is the normal case for the worst bugs.

## `merged.md` template

```markdown
# Merged findings: <task> / <round>

Sources that reported: C, X. Missing: none.

## Findings

### M1: <title>
- sources: C2, X1
- location: app/sync/worker.go:88
- severity: major
- confidence: 95
- lenses: bugs, impl
- trigger: <…>
- fix: <…>

<body>

## Open questions

- C3?: <question> (app/config.go:41)

## Pre-existing

- X4: <title> (app/cache.go:12), <severity>
```

Number `M` ids most severe first. Every `M` finding lists the finder ids it came from, since that is
how the report shows which models agreed.

**Example.** `C2` (major, 70, worker.go:88) and `X1` (minor, 85, worker.go:89) describe the same
missing reset. They become `M1`: severity major, confidence `min(99, 85 + 10) = 95`, sources `C2, X1`.
