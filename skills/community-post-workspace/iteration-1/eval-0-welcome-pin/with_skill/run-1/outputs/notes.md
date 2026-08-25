# Judgment calls for the #wtr welcome pin

## Grounding (verified in the repo)

- Released state: `master` HEAD is exactly tag `v0.2.2`, which is also the
  latest GitHub Release (checked with `git describe --tags` and
  `gh release list`). Everything the README describes is shipped, so nothing
  in the pin needed a "coming soon" hedge.
- The install command `brew install abogoyavlensky/tap/wtr` and the mise
  option are copied from the README verbatim. I did not run the brew install
  cold in this environment, so it is technically unverified end-to-end; it is
  the same command the README and release pipeline advertise.
- All commands in the try-it snippet (`create --sh`, `remove`, bare `wtr`
  dashboard) exist in v0.2.2 per the README and command table.

## Tone and shape

- Audience assumed to be developers who know git but not necessarily git
  worktrees, so the hook explains the payoff (one task = one worktree, no
  path juggling) rather than leading with "worktree manager".
- Mentioned agents in one clause ("several agents or tasks each need their
  own checkout") because the README's own quickstart frames it that way and
  it is the strongest 2026 hook for a dev Discord. Easy to cut if the server
  is not agent-oriented.
- Kept the pin to the welcome/reference shape (~10 lines): pitch, try-it,
  links, what-to-post. No feature list; the README carries that.
- Left out: `switch` (detached-HEAD nuance needs explaining), completions,
  config, and the let-go/lgx build story. They would dilute a pin; the repo
  link covers them. The bare `wtr` dashboard got one line because it is the
  best "wow" for zero extra typing.
- Maturity note: v0.2.x, so "pre-1.0, rough edges expected, reports are
  gold" to convert complaints into contributions.
- Two emoji total (👋 header, 🙏 close) plus 🔗 as a link anchor - structural
  only, per Discord conventions.

## Ready answer for the predictable question

"Why doesn't `wtr` just cd me into the worktree?" - A child process can't
change its parent shell's directory, so `wtr create --sh` / `wtr run <name>`
open a subshell in the worktree instead; `exit` returns you with the shell's
exit code. I deliberately left this out of the pin; use this one-liner when
it comes up.

## Paste gotchas

- The pin contains a code block. If you copy it from a fenced markdown
  wrapper, Discord ends the outer fence at the first inner ``` - paste the
  file's raw content directly and it renders fine.
- Repo/releases links are wrapped in `<...>` to suppress Discord's link
  embeds in the pin. Remove the angle brackets if you want the embed card.
