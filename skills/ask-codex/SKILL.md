---
name: ask-codex
description: Consult the Codex CLI (`codex exec`) as an independent second brain when the current session is stuck on a task or problem — a bug that survives repeated fix attempts, an error you can't explain, a design question you keep going back and forth on. Use when the user says "ask codex", "what does codex think", "get a second opinion", or names codex in any problem-solving context. Also use PROACTIVELY without being asked: if you have made 2-3 distinct attempts at the same problem and it still fails, or you notice you are cycling through hypotheses without new evidence, consult codex before trying again. Not for code review — that's the `review-with-codex` skill.
---

# Ask Codex

Sends a well-formed problem statement to the Codex CLI running in the
background, then brings its answer back into this session as a second
opinion. Codex is a different model with different blind spots — when this
session is stuck, a fresh, independent look often breaks the loop faster
than another attempt along the same line of thinking.

Codex is consulted **for analysis only**: it explores the repo and runs
read commands, and its prompt instructs it not to edit anything. You
evaluate its advice and apply what holds up.

**Announce at start:** "I'm using the ask-codex skill to get a second
opinion from Codex." If triggering proactively (not user-requested), say
why in one line, e.g. "Three fixes for this test failure haven't worked,
so I'm asking Codex for a fresh look."

## When to trigger proactively

Being stuck is a state you can detect in yourself. Signals:

- You've tried 2-3 **distinct** fixes for the same failure and it persists.
- You're re-reading the same files hoping to see something new.
- Your current hypothesis is a variation of one that already failed.
- An error message contradicts what the code plainly says should happen.

Consulting codex at that point is cheap (it runs in the background while
you continue gathering evidence) and the alternative — another lap around
the same loop — is usually more expensive. Don't wait for the user to
suggest it.

## Workflow

### 1. Compose the stuck-report

Write the prompt to a file — it's multi-line and quoting it inline is
fragile:

```bash
mkdir -p .tmp
TS=$(date +%s)
PROMPT=.tmp/codex-ask-${TS}-prompt.md
OUT=.tmp/codex-ask-${TS}-answer.md
LOG=.tmp/codex-ask-${TS}.log
```

Codex runs in this same working directory and can explore the repo itself,
so give it *pointers and evidence*, not file dumps. A good stuck-report
has five parts:

```markdown
## Goal
What I'm ultimately trying to achieve, in one or two sentences.

## Problem
The specific failure. Include the exact error output, verbatim.

## What I've tried
Each distinct attempt and what happened. This matters most — it stops
codex from re-suggesting things that already failed.

## Relevant code
File paths (with line numbers where useful) to start from. Mention
anything non-obvious about the setup that the repo won't reveal.

## Question
The specific question. "Why does X happen when Y?" beats "help".

---
This is a consultation only: explore the repo and run read commands as
needed, but do NOT modify, create, or delete any files.
```

Honesty in "What I've tried" is what makes the second opinion valuable —
include attempts that feel embarrassing in hindsight. If you're wrong
about something, codex can only catch it if you show your reasoning.

### 2. Invoke codex in the background

Run via the Bash tool with `run_in_background: true`. The harness notifies
you on completion — do not poll.

```bash
codex exec \
  --skip-git-repo-check \
  --dangerously-bypass-approvals-and-sandbox \
  -o "$OUT" \
  - < "$PROMPT" > "$LOG" 2>&1
```

- `-` reads the prompt from stdin (avoids shell-quoting a long prompt).
- `--dangerously-bypass-approvals-and-sandbox` — do NOT use `-s read-only`:
  its bwrap sandbox fails to start in containerized environments
  (`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`), and
  even where it works, codex can block on an approval prompt with no TTY.
  The prompt must therefore state explicitly that codex is consulted for
  analysis only and must not modify any files.
- `-o` writes codex's final answer to `$OUT`; `$LOG` keeps the full
  transcript for debugging.
- **Leave `-m` unset** — codex uses the model from `~/.codex/config.toml`.
  Only pass it if the user explicitly asks.

> **Flag compatibility — verify before assuming.** Codex CLI flags vary by
> version (checked against `codex-cli 0.142.5`). If the run exits with
> code 2 and an "unexpected argument" message in `$LOG`, run
> `codex exec --help`, drop or swap the offending flag, and re-invoke —
> don't retry the same command.

Tell the user: "Codex is looking at it in the background — I'll surface
its take when it finishes." Then **keep working**: gather more evidence,
add logging, write a minimal reproduction. Don't sit idle waiting, and
don't make large speculative edits that a codex answer might invalidate.

### 3. When codex completes

1. **Exit code ≠ 0** → read `$LOG`, surface the error (common causes:
   codex auth lapsed, flag mismatch for the installed version, network).
   Fix and re-invoke once; if it fails again, tell the user and continue
   without the second opinion.
2. **Exit code = 0** → read `$OUT`.
3. **Evaluate before applying.** Codex is a second opinion, not an oracle
   — verify its claims against the actual code the same way you'd check
   your own hypothesis. Then tell the user, clearly attributed:
   - What codex thinks the problem is (quote the key part).
   - Whether you agree, and why — especially where you disagree.
   - What you'll do next.
4. If the diagnosis holds up and you're mid-task, apply the fix and verify
   it the way the original task demands. If the user only asked for a
   consultation, report and stop.

### 4. Follow-ups (max ~2 rounds)

If the answer doesn't unstick things but is *engaging with the right
problem*, follow up in the same codex session — it keeps codex's
exploration context, so don't re-explain from scratch. Send only what's
new: what you tried based on its advice, and the new evidence.

```bash
codex exec resume --last \
  --skip-git-repo-check \
  --dangerously-bypass-approvals-and-sandbox \
  -o "$OUT2" \
  - < "$FOLLOWUP_PROMPT" > "$LOG2" 2>&1
```

Stop after about two follow-ups. If codex is off-track by then, more
rounds rarely converge — tell the user both models are stuck, summarize
the combined state of knowledge (what's ruled out, what's still open),
and ask how they want to proceed. That summary is itself valuable output.

### 5. Cleanup

Leave `.tmp/codex-ask-*` files in place — they age out naturally and the
prompt/answer pair is useful for debugging a bad consultation.

## What this skill is not

- **Not code review.** "What does codex think of my changes" → use
  `review-with-codex` (`codex exec review`).
- **Not delegation.** Codex advises; the fix is applied and verified in
  this session, by you.
- **Not a replacement for evidence-gathering.** If you haven't yet read
  the failing code or reproduced the error, do that first — a stuck-report
  without evidence gets a generic answer back.
