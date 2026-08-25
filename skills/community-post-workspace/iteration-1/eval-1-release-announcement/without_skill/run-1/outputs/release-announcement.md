**wtr v0.2.2 is out** 🚢

A small patch release focused on shell completion:

- **Flags now complete after positional arguments.** Previously, typing something like `wtr remove feature-x --<TAB>` suggested nothing — now it correctly offers `--force`, `--base-dir`, and friends. Same for `wtr switch`. (Fixed via the tiny-cli v0.2.2 bump.)

**Upgrade:**
```
brew upgrade wtr
```
or with mise:
```
mise use -g github:abogoyavlensky/wtr@latest
```

Release: <https://github.com/abogoyavlensky/wtr/releases/tag/v0.2.2>

Found a bug or have an idea? Drop it here or open an issue. Happy worktree hopping! 🌳
