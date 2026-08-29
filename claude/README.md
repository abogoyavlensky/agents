# Claude Code settings

`settings.json` as used by the `agent` user on the dev servers. Provisioned
boxes are set up by [`../server/`](../server/); this is the Claude-side config
that goes on top.

## Use it

Copy (safest — Claude Code writes to this file, e.g. when you change theme via
`/config`, and a symlink would push those edits into the repo):

```bash
mkdir -p ~/.claude && cp claude/settings.json ~/.claude/settings.json
```

Or symlink, if you want the box and the repo to stay in lockstep and don't mind
`git status` noise:

```bash
ln -sfn "$PWD/claude/settings.json" ~/.claude/settings.json
```

Merge into an existing file rather than clobbering it:

```bash
jq -s '.[0] * .[1]' ~/.claude/settings.json claude/settings.json > /tmp/s.json \
  && mv /tmp/s.json ~/.claude/settings.json
```

## What's in it, and why

| Key | Why |
|---|---|
| `attribution.commit` / `.pr` = `""` | No `Co-Authored-By` or session trailers — matches the repo's commit convention |
| `permissions.defaultMode` = `auto` | Auto mode; fewer prompts on a box that exists to run agents |
| `permissions.allow` = `Bash(codex exec:*)` | Lets the `review-with-codex` and `ask-codex` skills run. Covers `codex exec`, `exec review`, `exec resume` |
| `model` / `effortLevel` | Opus 5 at high effort |
| `tui` = `fullscreen` | Flicker-free renderer |
| `autoMemoryEnabled` = `false` | No auto-memory on shared agent boxes |

## Notes

**The `Bash(codex exec:*)` rule must be applied by a human.** Claude Code's
classifier refuses to let an agent write its own permission rules — through
Bash *and* Edit, and being root doesn't help, because it gates the action rather
than the file. That is deliberate: an agent that can widen its own permissions
can grant itself anything. Use `/permissions` in-session, or `jq` it in yourself.

**It's needed because the codex skills hard-code
`--dangerously-bypass-approvals-and-sandbox`**, which the classifier otherwise
denies. That flag is deliberate, not a bug — the Lima sandbox can't run bwrap,
so swapping in `--sandbox read-only` would break that environment. See
`../sandbox/fresh_server_setup.md` Step 10.

**No secrets live here.** Credentials are in `~/.claude/.credentials.json`, which
is never committed.
