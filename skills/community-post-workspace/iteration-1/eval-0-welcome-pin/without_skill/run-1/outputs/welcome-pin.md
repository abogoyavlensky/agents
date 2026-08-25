# Welcome to #wtr 👋

**wtr** is a small CLI for managing git worktrees — the [w]ork[t]ree [r]outer. Working from your main repo, it frees you from remembering absolute paths, runs commands in any worktree, and lets you peek at a branch without disturbing it. One task, one worktree, one branch.

**Repo:** <https://github.com/abogoyavlensky/wtr>
**Latest release:** v0.2.2 — <https://github.com/abogoyavlensky/wtr/releases>

## Install

```sh
# Homebrew (macOS or Linux)
brew install abogoyavlensky/tap/wtr

# or with mise
mise use -g github:abogoyavlensky/wtr@latest
```

Prebuilt binaries are also on the releases page.

## The 60-second tour

```sh
wtr create --sh feature-x   # new worktree + branch, jump into a shell there
wtr run feature-x npm test  # run a one-off command in it, no cd
wtr switch feature-x        # inspect that branch from your main dir (detached)
wtr list                    # see every worktree at a glance
wtr                         # interactive dashboard: enter/c/s/d to jump, create, switch, remove
wtr remove feature-x        # done? drop the worktree and its branch
```

`wtr completion bash|zsh|fish` gets you dynamic completions, including worktree names. Full command docs live in the [README](https://github.com/abogoyavlensky/wtr#commands).

## What this channel is for

- ❓ Questions about using wtr and worktree workflows — no question too small
- 🐛 Bug reports — a quick message here is fine, and issues are welcome: <https://github.com/abogoyavlensky/wtr/issues>
- 💡 Feature ideas and feedback — especially on the interactive dashboard, which is new in 0.2.x
- 🔧 Contributions — PRs welcome; say hi here if you want to pick something up

When reporting a problem, please include your OS, `wtr` version, and the command you ran — it makes things much faster to track down.

Glad you're here — introduce yourself and tell us how you juggle your worktrees today!
