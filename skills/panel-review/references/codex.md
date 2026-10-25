# Running codex as a finder

Read this before launching the `X` finder. Codex gets the same composed prompt as the Claude finders,
on stdin, through plain `codex exec`. Not `codex exec review`: that one uses codex's built-in review
prompt, which would ignore our lenses, severity bar and template. It also refuses a custom prompt
together with `--uncommitted` or `--commit`.

## The command

Run it through the Bash tool with `run_in_background: true`. The harness notifies you when it exits,
so do not poll.

```bash
codex exec --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox -o .tmp/review/<task>/<round>/finder-X.md - < .tmp/review/<task>/<round>/prompt-X.md > .tmp/review/<task>/<round>/codex-X.log 2>&1
```

- `-o` writes codex's final message, which is the findings, straight to `finder-X.md`.
- `-` reads the prompt from stdin.
- Leave `-m` unset. Codex uses the model in `~/.codex/config.toml`; pass `-m` only when the user asks
  for a specific model.

**Start the command with `codex` and write the paths literally.** Hosts allow this with a prefix rule
such as `Bash(codex exec:*)`, which matches only when the command *begins* with that string. Prefix it
with anything (`mkdir -p … && codex …`, `cd repo && codex …`, `TS=… codex …`) and the rule stops
matching. The call then falls through to the permission classifier, which may refuse the bypass flag,
and that refusal looks like codex being broken. Do the setup (`mkdir`, composing the prompt) in an
earlier, separate Bash call. Shell variables do not survive between Bash calls anyway, which is the
other reason for literal paths.

## Why the bypass flag, not `-s read-only`

`-s read-only` looks like the responsible choice, but on many containers it does not work. Codex
sandboxes with bubblewrap, and when `bwrap` is missing or cannot set up a network namespace, every
command codex runs fails (`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`). The run then
exits 0 with a report saying it could not inspect anything. Codex also cannot answer an approval prompt
in a background run with no TTY, so it would hang. The bypass is safe here because the prompt forbids
writes and review is a reading task.

## Flag drift

Codex CLI flags change between versions. If a run exits 2 with "unexpected argument" or "cannot be
used with" in the log, run `codex exec --help`, fix or drop the offending flag, and relaunch **once**.
Do not keep retrying the same command.

## When codex fails

Codex is one finder of several. Its failure costs the round a second opinion, but it does not end the
review. Mark it missing and continue with the Claude finder(s) when any of these holds:

- the exit code is non-zero after the one flag-drift retry (auth lapsed, network, quota)
- `finder-X.md` is missing or empty
- `finder-X.md` says codex could not read the code or run commands (the broken-sandbox case). "No
  findings." and "I could not look" are different results, and only the first is evidence

Record `missing reviewer: X (<reason, quoted from codex-X.log or finder-X.md>)` in `merged.md` and
`report.md`, and say it in the brief. If every finder failed, the round is an error, not a clean
review. Report it that way.
