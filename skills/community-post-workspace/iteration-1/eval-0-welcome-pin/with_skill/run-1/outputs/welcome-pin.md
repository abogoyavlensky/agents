👋 **Welcome to #wtr**

**wtr** routes you between git worktrees so one task = one worktree = one branch, with no absolute paths to remember. Spin up a worktree and land in a shell there, run one-off commands in any worktree from the main repo, and clean up worktree + branch in one go. Handy when several agents or tasks each need their own checkout.

Try it in under a minute (macOS/Linux):
```
brew install abogoyavlensky/tap/wtr
wtr create --sh try-wtr   # new worktree + branch, shell opens there
exit                      # back where you were
wtr remove try-wtr        # worktree and branch, gone
```
Or just run `wtr` with no arguments for an interactive dashboard of your worktrees.

🔗 Repo: <https://github.com/abogoyavlensky/wtr> · Releases: <https://github.com/abogoyavlensky/wtr/releases> (also installs via mise)

Post here: questions, bug reports, workflows you've built around it, ideas. It's pre-1.0 and moving fast, so rough edges are expected and reports are gold. 🙏
