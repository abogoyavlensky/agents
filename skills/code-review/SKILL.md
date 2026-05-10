---
name: code-review
description: Review local code changes — uncommitted work, the current branch vs main, or a specific commit — for code quality, minimal implementation, basic security, and feature completeness. Use this skill whenever the user asks to review code, review changes, review a branch, review a commit, audit a diff, sanity-check what's about to be committed, or asks "is this ready to ship?" — even when they don't say the word "review" explicitly. This is a fast local review run inside the project, not the cloud-based /ultrareview. The output is a severity-grouped findings report with file:line citations.
---

# Code review

This skill produces a focused review of local changes against four axes: code
quality, minimal implementation, basic security, and feature completeness.
Findings are grouped by severity (Must fix / Should fix / Nit), with each
item tagged by axis and pointing to a specific file and line.

The reviewer reads the diff carefully, but also reads enough surrounding code
to judge intent and call sites. A diff in isolation is rarely enough.

## Workflow

### 1. Determine scope

Pick what to review based on the user's request and repo state. Resolve in
this order:

1. **User named a commit or range** (e.g. "review commit abc123",
   "review HEAD~3..HEAD"): use that explicitly with `git show <ref>` or
   `git diff <range>`.
2. **User said "branch", "this branch", "my branch"** or similar: diff the
   current branch against the main branch.
   - Determine the main branch: `git rev-parse --verify main` then
     `master` as fallback. If neither exists, ask.
   - Use `git log --oneline <main>..HEAD` and
     `git diff <main>...HEAD`.
3. **No explicit scope from the user**: auto-detect.
   - If there are uncommitted *tracked* changes, review those
     (`git diff HEAD`). Untracked files do not count toward scope
     detection and are not included in the review by default — they're
     usually local scratch (build outputs, editor configs, dev tooling
     like `.mise.toml` or `.clj-lsp/`, WIP that was never meant for
     review). If the user wants a specific untracked file reviewed,
     they'll name it.
   - Else if the current branch is ahead of main: review the branch
     (case 2).
   - Else: ask which commit or scope to review.

State the chosen scope at the top of the report so the user can correct it
if you guessed wrong.

### 2. Gather the diff and infer intent

For the chosen scope:

- Get the diff with enough context (`git diff -U10` or higher) so you can
  reason about callers and surrounding logic.
- For commit/branch scopes, read the commit message(s) — they're the stated
  intent. Quote the relevant phrasing in your head.
- For uncommitted scopes with no commit message, infer intent from the diff
  itself. If intent is genuinely unclear (e.g. unrelated changes mixed
  together, or a single change whose purpose isn't legible), say so in the
  report under Notes and continue with what you can reason about.

When a finding turns on something outside the diff (a caller, a related
file, a config), open that file and verify before reporting. Don't speculate
— either confirm or skip.

### 3. Review against the four axes

Each axis below has a short explanation of what to look for and why. These
aren't checklists to grind through mechanically — they're lenses. Read the
diff once, then ask yourself the questions in each section.

#### Code quality

Is the change clear to read? Does each function do one thing? Are names
unambiguous? Is error handling at the right boundary (system edges, not
internal calls between trusted modules)? Are there dead branches, unused
imports, copy-pasted blocks that drifted?

Flag anything a maintainer reading this six months from now will need to
re-derive. The cost of unclear code is paid every time someone touches it.

#### Minimal implementation

Did the change add only what the task required? Watch for:

- **Premature abstractions** — extracting a helper used once, generic
  parameters that have no second caller, type hierarchies for a single
  concrete case.
- **Speculative generality** — options/flags that aren't exercised, branches
  for hypothetical future inputs, "in case we ever need it" code.
- **Defensive checks against impossible states** — guards that protect
  against conditions the surrounding code already prevents.
- **Backwards-compatibility scaffolding** for code that hasn't shipped.
- **Restating-the-code comments** — comments that paraphrase the next line
  rather than capture a non-obvious why.

Three similar lines beats a premature abstraction. Code is easier to
generalize once a real second use arrives than to un-generalize when the
abstraction was wrong.

#### Basic security

You're not running a full security audit — you're catching obvious mistakes
that wouldn't survive review at any company. Look for:

- **Injection** — shell, SQL, HTML, logs — anywhere user input lands in a
  string that's then interpreted.
- **Path traversal** — user-supplied paths joined to a base directory
  without validation, or read/written without checking the resolved path.
- **Untrusted deserialization** — `eval`, `pickle`, `Marshal`, `JSON.parse`
  on attacker-controlled input.
- **Hardcoded secrets or credentials** in source.
- **Missing input validation at trust boundaries** — HTTP handlers,
  CLI args from external sources, file uploads.
- **Catastrophic regex** — unbounded backtracking on user input.

Internal code calling internal code is a trusted boundary; don't demand
validation between components that already trust each other. Validation
goes at the system edge.

#### Feature completeness

Does the change actually accomplish what its commit message (or apparent
intent) says? Look for:

- **Stated behavior not delivered** — message says "fix X under Y" but only
  X is touched.
- **Edge cases the change forgot** — empty input, nil/null, boundary values
  (zero, negative, max), error paths.
- **Call sites missed** — a renamed function whose other callers weren't
  updated; a new return shape one consumer doesn't handle.
- **Tests that don't actually exercise the new behavior** — tests added,
  but they pass with the old code too.
- **Docs/README/comments that no longer match** the new behavior.

If the intent is unclear, say so in Notes rather than inventing one.

### 4. Write the report

Use the template below. Severity rules of thumb:

- **Must fix** — bugs, security issues, broken feature completeness, things
  that will hurt someone (user, maintainer, prod).
- **Should fix** — clear improvements to quality or minimality that aren't
  blocking but should be addressed before merging.
- **Nit** — style, naming, minor wording. The reader can ignore these
  without consequence.

Keep findings short. One or two sentences per item is usually enough. The
file:line lets the reader jump there; don't restate the code.

If a section is empty, omit it. If the whole review finds nothing, say
that explicitly — a clean review is a useful signal too.

## Report template

```markdown
# Code review: <one-line scope description>

**Scope:** <e.g. "uncommitted changes — 4 files, +120 -34">
**Intent:** <quoted commit message phrase, or short inferred summary, or "unclear">

## Must fix

- **<axis>** `<file>:<line>` — <short title>
  <one or two sentence rationale>

## Should fix

- **<axis>** `<file>:<line>` — <short title>
  <one or two sentence rationale>

## Nit

- **<axis>** `<file>:<line>` — <short title>
  <one or two sentence rationale>

## Notes

<anything else worth surfacing — e.g. "no security-sensitive surfaces touched",
"tests added cover the change", "two unrelated changes mixed in this diff",
or omit if there's nothing to say>
```

Axis tags: `quality`, `minimal`, `security`, `completeness`.

## Don't propose fixes in the report

Report findings, not patches. The user will decide what to act on and ask
for fixes in follow-up turns. Adding fixes inline turns a review into an
implementation pass and obscures what was found.

If a fix is genuinely two characters (a typo in a string, a missing close
paren) and pasting it is shorter than describing it, you can include it —
but that's the exception.

## What this skill is not

- **Not /ultrareview.** That's a separate cloud-based multi-agent review for
  branches/PRs. This skill is local, fast, and runs inline in the
  conversation.
- **Not a full security audit.** It catches obvious mistakes; it doesn't
  replace a real security review for sensitive code paths.
- **Not auto-fix.** Findings only.
- **Not a substitute for tests.** A passing review doesn't mean the code is
  correct — only that the diff doesn't have visible defects.
