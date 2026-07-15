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

Save locally in the `.tmp` dir in the current repo:

```bash
TS=$(date +%s)
OUT=.tmp/codex-review-${TS}.md      # final review message
LOG=.tmp/codex-review-${TS}.log     # full stdout/stderr for debugging
```

### 3. Invoke codex in the background

Run via the Bash tool with `run_in_background: true`. The harness will
notify you on completion — do not poll.

```bash
codex exec review \
  --skip-git-repo-check \
  --dangerously-bypass-approvals-and-sandbox \
  <SCOPE_FLAG> \
  -o "$OUT" \
  > "$LOG" 2>&1
```

`--dangerously-bypass-approvals-and-sandbox` is what makes a background run
work: `codex exec review` is read-only, but without it codex can block on an
approval/sandbox prompt with no TTY and the background job hangs until timeout.
Safe here precisely because the review never writes.

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
> version (checked against `codex-cli 0.135.0`). Two gotchas seen in practice:
> - `--color never` is **rejected** (`unexpected argument '--color'`). Don't
>   pass it. `-o` already writes a clean final message; ANSI in `$LOG` is
>   harmless.
> - A positional `PROMPT` **cannot** be combined with `--uncommitted` or
>   `--commit` (`the argument '--commit <SHA>' cannot be used with '[PROMPT]'`,
>   seen on codex-cli 0.142.x). Treat scope flags as prompt-incompatible by
>   default; only `--base` is known to accept one.
>
> If a run fails with exit code 2 and an "unexpected/incompatible argument"
> message in `$LOG`, run `codex exec review --help`, drop or swap the offending
> flag, and re-invoke — don't keep retrying the same command.

Tell the user: "Codex review started in the background (scope: …). I'll
surface findings when it finishes." Then continue with whatever's next —
the caller decides whether to block or carry on.

### 4. When codex completes

When the background bash notifies completion:

1. **Exit code ≠ 0** → read `$LOG`, surface the error to the user, and
   suggest a fix (most common causes: codex auth lapsed, wrong flag for the
   installed codex version, repo not git-initialised).
2. **Exit code = 0** → read `$OUT`. That's codex's final review message.
3. Summarise findings for the user, **grouped by severity** (must fix /
   should fix / nit) with file:line citations when codex provided them.
   Quote codex's text where possible — it's a second opinion, not your own.
4. Ask the user how to handle each finding. If invoked from
   `executing-plans`, fold must-fix items into the current task before
   moving on.

### 5. Cleanup

Leave `.tmp/codex-review-*` files in place — they age out naturally and are
useful for debugging a bad review run.

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
