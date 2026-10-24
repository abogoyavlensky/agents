---
name: panel-review
description: Structured multi-model review of local work. A panel of Claude subagents and codex reviews the same subject with the same prompt, their findings are merged, and a fresh verifier re-checks every finding before it is reported. Reviews uncommitted changes, the current branch vs main, a commit or range, or a plan, design or spec document. Modes - focused (default; bugs and goal-fit on Claude and codex), full (every lens - bugs, impl, quality, minimal, security, docs, tests, adversarial), plan (for documents). Use this whenever the user asks to review code, changes, a branch, a commit or a plan, to audit a diff, for a second opinion or what codex thinks of the changes, or asks "is this ready to ship" or "anything wrong with this" - even without the word review. It is also the review checkpoint that executing-plans, makegoal and fastplan run. Local and read-only; not the cloud /code-review ultra.
---

# panel-review

A review from one model in one pass varies with whatever that model looked at that time, and nothing
checks its findings. This skill makes a review repeatable instead:

1. **Find.** Several finders review the same subject in parallel: Claude subagents and codex, each
   carrying named *lenses*. They get identical prompts built from the files in `references/`, so two
   runs ask the same questions against the same severity bar.
2. **Merge.** You dedupe the findings and raise confidence where independent finders agree.
3. **Verify.** Fresh subagents re-check every finding against the code and return one verdict each.
   This is what makes the output trustworthy: a finding the verifier rejects never reaches the user.
4. **Report.** You write `report.md` and either brief the user or hand a gating list back to the
   calling skill.

All work happens in a round folder in the reviewed repo, `.tmp/review/<task>/<NN-name>/`, which keeps
the composed prompts, raw finder output, merge, verdicts and report. When a review looks wrong, that
folder shows why.

## Modes

| mode | finders (id prefix: lenses) | verifiers |
|---|---|---|
| `focused` (default) | `C`: bugs, impl (Claude) · `X`: bugs, impl (codex) | 1 |
| `full` | `B`: bugs, impl · `Q`: quality, minimal, security · `D`: docs, tests (all Claude) · `X`: adversarial (codex) | 1 per top-level directory, max 3 |
| `plan` | `C`: plan, grounding (Claude) · `X`: plan, grounding (codex) | 1 |

In `focused` and `plan` both finders carry the same lenses on different models, so when they agree,
the agreement is real corroboration. Use `full` when the user says "full", "thorough", "everything" or
"before merge". Use `plan` whenever the subject is a document, not a diff. Otherwise use `focused`.

Lens files live in `references/lenses/`. Code modes use `references/preamble-code.md`; plan mode uses
`references/preamble-plan.md`.

## Step 1: Resolve the scope

Pick what to review from the request and the repo state, in this order:

1. **A document path** (a plan, design or spec): plan mode.
2. **A named commit or range**: `git show <sha>` or `git diff <range>`.
3. **"Branch", "this branch", "my branch"**: the branch against main. Find main with
   `git rev-parse --verify main`, falling back to `master`; if neither exists, ask.
4. **Nothing explicit**: uncommitted changes if there are any, meaning tracked changes (`git diff HEAD`)
   or untracked files git does not ignore (`git ls-files --others --exclude-standard`). Else the
   branch if it is ahead of main; else ask.

`git diff` never shows untracked files, so whenever the scope covers the working tree, list the
untracked, non-ignored files in `scope.md` under `new files (read in full)`. Otherwise new modules
reach no finder and the review passes on code nobody read. Name them in the scope line too, so the
user can drop local scratch from it. State the chosen scope and mode in one line before launching, so
a wrong guess costs one correction rather than a whole review.

**Record a fixed base in round 1** so later rounds review the original change together with its
fixes, not the fixes alone:

| scope | base | round 1 diff | later rounds |
|---|---|---|---|
| commit `<sha>` | `<sha>^` | `git diff <sha>^ <sha>` | `git diff <base>` |
| range `<a>..<b>` | `<a>` | `git diff <a> <b>` | `git diff <base>` |
| branch | `git merge-base <main> HEAD` | `git diff <base> HEAD` | `git diff <base>` |
| uncommitted | `git rev-parse HEAD` | `git diff <base>` | `git diff <base>` |

Write the base as a full sha. `git diff <base>` compares against the working tree, so it covers fixes
whether or not they were committed.

## Step 2: Prepare the round

Pick a task name: the branch name, `commit-<shortsha>`, or the plan's file name without extension,
slugified. A calling skill may pass its own, such as `<branch>-task3`. Number rounds `01-initial`,
`02-after-fix`, `03-after-fix`. Never write into a round that already has a `report.md`; open the next
one instead, so every review stays on record.

In the round folder, write:

- **`scope.md`**: the subject in one line, the base, the exact diff command (or the document's
  absolute path), and the `git log --oneline` command for the commits involved.
- **`goal.md`** (optional): what the change is meant to do. Use commit messages, the plan's Goal line,
  or the user's own words. Leave it out when there is nothing to say; the impl lens lowers its
  confidence rather than inventing a goal.
- **`context.md`**:

  ```markdown
  # Round context
  - workdir: /abs/path/to/repo
  - scope: /abs/.../scope.md
  - goal: /abs/.../goal.md | none provided
  - profile: /abs/docs/review-profile.md | /abs/AGENTS.md | none provided
  - preamble: /abs/<skill-dir>/references/preamble-code.md
  - prior rounds: none | /abs/.../01-initial/report.md, …
  ```

  For the profile, use `docs/review-profile.md` if it exists, else `AGENTS.md` or `CLAUDE.md`, else
  `none provided`. From round 2 onward, append this paragraph so finders do not just confirm the last
  round:

  > Earlier rounds of this review are listed above. Re-evaluate everything independently. A prior
  > round reporting an issue is not evidence that it is real, and a prior round missing one is not
  > evidence that it is absent.

Then build every finder's prompt in **one** Bash call, concatenating the preamble, the finder's lenses,
the finding template and the context, plus the finder's id prefix. `<skill-dir>` is the base directory
printed when this skill loaded. For focused mode:

```bash
R=.tmp/review/<task>/<round>; S=<skill-dir>/references
for P in C X; do { cat $S/preamble-code.md $S/lenses/bugs.md $S/lenses/impl.md $S/finding-template.md $R/context.md; printf '\nYour finding id prefix: %s\n' "$P"; } > $R/prompt-$P.md; done
```

For other modes, use the same shape with that mode's preamble and give each finder the lenses from
the Modes table. In full mode, for example, `prompt-Q.md` is `preamble-code.md` + `quality.md` +
`minimal.md` + `security.md` + the template + the context. In plan mode, both finders get
`preamble-plan.md` + `plan.md` + `grounding.md`, and `context.md` names `preamble-plan.md`.

Building prompts from files, not retyping them, is what keeps every finder's instructions identical,
and the composed `prompt-<P>.md` stays on record.

## Step 3: Find

Launch every finder **in one message**, so they run at the same time and none sees another's output:

- **Claude finders**: Agent tool, `subagent_type: general-purpose`, `run_in_background: true`, with the
  prompt: *"Read /abs/.../prompt-C.md and follow it exactly. Return only the findings, in the template
  it gives."*
- **Codex finder**: read `references/codex.md` and follow it. It covers the exact command, why it must
  start with `codex`, the sandbox flag and what counts as a failure.

Wait for every completion notification; do not poll. Write each Claude finder's final message
verbatim to `finder-<P>.md`. Codex writes `finder-X.md` itself.

When codex fails, continue with the Claude finders and record `missing reviewer: X (<reason>)`.
A missing second opinion weakens the round, but it is still a review. If **every** finder failed,
stop and report that as an error. An empty review must never pass as a clean one.

## Step 4: Merge

Read `references/merge.md` and follow it to write `merged.md`. It is bookkeeping: split out open
questions and pre-existing findings, dedupe, apply the confidence formula, take the highest severity,
drop nothing, and judge nothing. If no findings remain, skip verify and go to Step 6.

## Step 5: Verify

Split the merged findings into groups: one group in focused and plan mode; in full mode, one group per
top-level directory, merging the smallest ones until there are at most three. A verifier that sees
only its own group cannot anchor on a neighbouring finding.

For each group `<n>`, write its `M` blocks verbatim to `verify-group-<n>.md`, then compose the
prompt and launch the verifiers in one message:

```bash
R=.tmp/review/<task>/<round>; S=<skill-dir>/references
for n in 1; do cat $S/verify.md $R/context.md $R/verify-group-$n.md > $R/verify-prompt-$n.md; done
```

Each verifier is an Agent `general-purpose` subagent with `run_in_background: true` and the prompt:
*"Read /abs/.../verify-prompt-1.md and follow it exactly. Return only the verdict blocks."* Write its
final message to `verify-<n>.md`. When a verifier fails or returns no block for an id, relaunch it
once for the missing ids. The round is not finished yet, so this stays inside the same round and
costs no round from the cap. If it fails again, mark those findings `unverified` and record
`missing verifier: <n> (<reason>)`. Never drop the findings.

## Step 6: Report and present

Apply the verdicts (a `refined` verdict's corrected fields replace the originals), then read
`references/report.md` and follow it. It defines what gates, the `report.md` template, and the two
ways to finish:

- **The user asked for the review**: a decision brief that ends with `AskUserQuestion` about what to
  fix. If `docs/review-profile.md` is missing and `.tmp/review/.profile-declined` does not exist, add
  a second question in the same call offering to draft one from
  `references/review-profile-template.md`. On a decline, create `.tmp/review/.profile-declined` so
  you never ask again in that repo.
- **Another skill called it**: return the short gating block and stop. Ask nothing.

Fix nothing during the review. Fixes happen only after the user (or the calling skill) decides, and
they are checked by the next round.

## Rounds

A fix can be wrong or incomplete, so after fixing gating findings, run another round on the same task
with the same base. Each new round gets the earlier reports, with the guard paragraph telling finders
to judge independently.

- **Gating** means `critical` or `major` with a `confirmed`, `refined` or `unverified` verdict.
  Nothing else gates.
- **Stop** at the first of:
  - a round with nothing gating (clean);
  - the gating count is not strictly lower than the previous round's **and** a gating finding repeats.
    The fix is not landing, so say which finding and stop;
  - **3 rounds**: report what is still open.
- Before committing a fix, list what the fix touched: every enum value or case it branches on, every
  error path, every caller of a changed signature. Check that the fix addresses the mechanism the
  finding named, not just the example it gave. Fixes that only patch the example are where the next
  round's findings come from.
- Never push. Pushing stays the user's decision, and leaving it to them keeps every round revertible.

## What this skill is not

- **Not auto-fix.** Finders and verifiers are read-only; fixing is a separate, chosen step.
- **Not `/code-review ultra`.** That is the cloud multi-agent review; this one runs locally.
- **Not a security audit.** The security lens catches obvious mistakes, not a threat model.
- **Not a test run.** A clean review means the panel found no defect by reading, not that the code
  works.
