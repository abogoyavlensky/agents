📦 **wtr v0.2.2 is out**

Small quality-of-life release: flags no longer have to come before arguments.

```bash
wtr remove feature-x --force   # now works (was: wtr remove --force feature-x)
```

Tab completion follows along: `wtr remove feature-x -<tab>` now suggests `--force`. Completions call back into the binary, so upgrading is enough, no need to regenerate your completion script.

One deliberate exception: `wtr run`, where everything after the worktree name belongs to your command. `wtr run feature-x npm test --watch` still hands `--watch` to npm, exactly as before.

Upgrade:

```bash
brew upgrade abogoyavlensky/tap/wtr
# or with mise: mise up
```

🔗 Release notes: <https://github.com/abogoyavlensky/wtr/releases/tag/v0.2.2>
