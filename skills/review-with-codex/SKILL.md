---
name: review-with-codex
description: Run a second-opinion code review using the Codex CLI (`codex exec review`) in the background and surface the findings when it finishes. Use when the user says "review with codex", "second-opinion review", "what does codex think", or wants an outside model to look at local changes (uncommitted, current branch vs main, or a named commit). Also used as the review checkpoint inside the `executing-plans` skill.
---

# Review with Codex

Delegates a code review to the Codex CLI, in the background, and brings the
report back into the current session. This is a *second opinion* from a
different model — useful alongside or in place of the inline `code-review`
skill when you want independent verification.

**Announce at start:** "I'm using the review-with-codex skill to get a
second-opinion review."

## Workflow

### 1. Resolve scope

Mirror the `code-review` skill's resolution so behavior stays consistent:

1. **User named a commit** ("review commit abc123") → `--commit <sha>`.
2. **User named a base branch** ("review against develop") → `--base <branch>`.
3. **User said "branch" / "this branch"** → `--base <main-branch>`.
   - Determine main with `git rev-parse --verify main`, fallback to `master`.
     If neither exists, ask.
4. **No explicit scope** → auto-detect:
   - Tracked uncommitted changes (`git diff HEAD --quiet || echo dirty`) →
     `--uncommitted`.
   - Else current branch is ahead of main → `--base <main-branch>`.
   - Else ask which scope to review.

State the chosen scope to the user before invoking codex so they can correct
a wrong guess.

### 2. Pick output paths

Save locally in the `.tmp` dir in the current repo. **Choose literal paths and
write them into the command by hand** — no `$OUT`, no `TS=$(date +%s)`:

```
.tmp/codex-review-<scope>.md    # final review message
.tmp/codex-review-<scope>.log   # full stdout/stderr for debugging
```

Pick `<scope>` so runs don't collide — the commit sha, the task number, or the
branch name (`.tmp/codex-review-task2.log`, `.tmp/codex-review-5a57a02.log`).

Two reasons this isn't the shell-variable version it looks like it should be:

- **Bash calls don't share state.** Each one is a fresh shell, so a `TS=`
  assigned in one call is empty in the next. The variables only ever worked
  because they were chained into a single `&&` command — which is exactly what
  breaks the next point.
- **The invocation has to *begin* with `codex`.** See step 3.

Run `mkdir -p .tmp` if the directory is missing, folded into the git commands
you already ran for scope resolution — not into the codex call.

### 3. Invoke codex in the background

Run via the Bash tool with `run_in_background: true`. The harness will
notify you on completion — do not poll.

> **The command must start with `codex`, as a single bare command.** Hosts
> allowlist this skill with a prefix rule — `Bash(codex:*)` or
> `Bash(codex exec:*)` — that matches only when the command *begins* with that
> string. Prefix it with anything (`mkdir -p .tmp && codex …`,
> `TS=$(date +%s) && codex …`, `cd repo && codex …`) and the rule stops
> matching, the call falls through to whatever permission classifier the host
> applies, and `--dangerously-bypass-approvals-and-sandbox` is a plausible
> thing for that classifier to refuse. The denial reads as "codex is broken";
> it isn't. Do setup in a **separate** Bash call. A trailing redirect is fine —
> the command still begins with `codex`.

```bash
codex exec review \
  --skip-git-repo-check \
  --dangerously-bypass-approvals-and-sandbox \
  <SCOPE_FLAG> \
  -o .tmp/codex-review-<scope>.md \
  > .tmp/codex-review-<scope>.log 2>&1
```

`--dangerously-bypass-approvals-and-sandbox` is what makes a background run
work: `codex exec review` is read-only, but without it codex can block on an
approval/sandbox prompt with no TTY and the background job hangs until timeout.
Safe here precisely because the review never writes.

It is also, on many machines, the only mode that runs at all — read *Sandbox
modes* below before reaching for `-s read-only` as a safer-looking substitute.

Where `<SCOPE_FLAG>` is exactly one of:

- `--uncommitted`
- `--base <main-branch>`
- `--commit <sha>`

Optional positional prompt at the end for focused instructions, e.g.
`"Pay special attention to the new auth middleware in pkg/auth/"`. Keep it
short; codex has a built-in review prompt. **Caveat (see flag compatibility
below): recent codex versions reject a positional PROMPT together with
`--uncommitted` AND with `--commit`.** Only `--base` is known to accept one;
otherwise drop the prompt and rely on codex's built-in review prompt.

> **Flag compatibility — verify before assuming.** Codex CLI flags vary by
> version. Gotchas seen in practice:
> - A positional `PROMPT` **cannot** be combined with `--uncommitted` or
>   `--commit` (`the argument '--commit <SHA>' cannot be used with '[PROMPT]'`,
>   seen on codex-cli 0.142.x). Treat scope flags as prompt-incompatible by
>   default; only `--base` is known to accept one.
> - `--color never` was **rejected** on codex-cli 0.135.0 (`unexpected argument
>   '--color'`) and **accepted** on 0.151.0. You never need it — `-o` already
>   writes a clean final message and ANSI in the log is harmless — so drop it at
>   the first complaint rather than checking.
>
> If a run fails with exit code 2 and an "unexpected/incompatible argument"
> message in the log, run `codex exec review --help`, drop or swap the offending
> flag, and re-invoke — don't keep retrying the same command.

Tell the user: "Codex review started in the background (scope: …). I'll
surface findings when it finishes." Then continue with whatever's next —
the caller decides whether to block or carry on.

### 4. When codex completes

When the background bash notifies completion:

1. **Exit code ≠ 0** → read the `.log`, surface the error to the user, and
   suggest a fix (most common causes: codex auth lapsed, wrong flag for the
   installed codex version, repo not git-initialised).
2. **Exit code = 0** → read the `.md`. That's codex's final review message.
   Read it before reporting: a broken sandbox (see *Sandbox modes*) exits 0
   with a report saying it reviewed nothing. "No findings" and "could not
   look" are not the same result, and only one is worth relaying as
   reassurance.
3. Summarise findings for the user, **grouped by severity** (must fix /
   should fix / nit) with file:line citations when codex provided them.
   Quote codex's text where possible — it's a second opinion, not your own.
4. Ask the user how to handle each finding. If invoked from
   `executing-plans`, fold must-fix items into the current task before
   moving on.

### 5. Cleanup

Leave `.tmp/codex-review-*` files in place — they age out naturally and are
useful for debugging a bad review run.

## Sandbox modes

`-s read-only` looks like the responsible choice for a review. On many
containers it does not work at all: codex sandboxes with **bubblewrap**, and if
`bwrap` is missing from PATH — or present but unable to configure a network
namespace — every command codex runs fails before it starts:

```
bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted
```

Codex handles this honestly rather than silently: it reports back that it could
not inspect anything and declines to invent findings. So the run *succeeds*
(exit 0) while the report says "unable to review" — check the report body, not
just the exit code.

Check with `command -v bwrap`. If it's absent and you can't install it (no
passwordless sudo), `--dangerously-bypass-approvals-and-sandbox` is the only
working mode. That is a safe trade *for this skill specifically*: review never
writes, and an agent host is already a sandbox.

## When codex can't run git at all

If the sandbox is broken and the bypass flag is unavailable too, codex can
still review — give it everything inline so it needs no shell:

```bash
codex exec -s read-only --skip-git-repo-check -o .tmp/codex-review-<scope>.md - \
  < .tmp/codex-review-<scope>.prompt.md \
  > .tmp/codex-review-<scope>.log 2>&1
```

Build the prompt file in an earlier Bash call from the `code-review` skill's
instructions plus the material codex would otherwise fetch itself: `git log`
for the scope, `git diff -U20`, and the handful of unchanged files a reviewer
needs for call sites and intent. Tell it explicitly that no shell is available
and to report what it cannot judge rather than guessing. This produces a real
review — it costs you the judgement about *which* surrounding files matter,
which codex would otherwise make for itself.

## Flags worth knowing

- `--uncommitted` — staged + unstaged + untracked changes. (Cannot be combined
  with a positional PROMPT on some versions — see flag compatibility above.)
- `--base <branch>` — diff HEAD against the branch.
- `--commit <sha>` — review just that commit.
- `--skip-git-repo-check` — allow running outside a strict git check.
- `--dangerously-bypass-approvals-and-sandbox` — required for unattended
  background runs (no TTY for approval prompts); safe because review is
  read-only.
- `-o <file>` — write the agent's final message to a file.
- `-m <model>` — override the codex model. **Leave unset by default**;
  codex uses what's configured in `~/.codex/config.toml`. Only pass if the
  user explicitly asks.
- `--json` — emit JSONL of events. Use only if you need streaming progress;
  `-o` already captures the final report.

## What this skill is not

- **Not the local `code-review` skill.** That one runs inline in this
  session. This delegates to a separate process and model.
- **Not auto-fix.** Codex is read-only; acting on findings is a follow-up
  step in the calling session.
