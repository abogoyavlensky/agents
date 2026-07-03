# Manual setup, README snippets, and cutting a release

The scaffolded workflow is self-contained, but three things live outside the
repo and must be done by the user (a human with access to the GitHub org). Relay
these clearly; you cannot do them from the CLI.

## 1. The Homebrew tap repository

`brew install <owner>/tap/<name>` resolves to the repo
`github.com/<owner>/homebrew-tap`, in its `Formula/` directory. The release
workflow pushes `Formula/<name>.rb` there on every release.

- If the user already publishes other tools this way (e.g. they have wtr/lgx in
  a tap), the repo exists — nothing to do; this tool just adds its own formula
  file alongside the others.
- If not, they create a **public** repo named exactly `homebrew-tap` under their
  account/org. It can start empty; the workflow creates `Formula/` on first run.

## 2. The `HOMEBREW_TAP_TOKEN` secret

The release workflow's `homebrew` job clones and pushes to the tap repo, which
`GITHUB_TOKEN` cannot reach (it's scoped to the current repo). So the user adds
a repository secret:

- **Name:** `HOMEBREW_TAP_TOKEN`
- **Value:** a GitHub Personal Access Token with write (`contents`) access to
  the `homebrew-tap` repo. A fine-grained PAT scoped to just that repo is ideal;
  a classic `repo`-scoped PAT also works.
- **Where:** the tool's repo → Settings → Secrets and variables → Actions →
  New repository secret.

`GITHUB_TOKEN` (used for creating the release itself) is provided automatically;
only the tap token is manual.

## 3. Cutting a release

Once the files are in and the secret is set:

```bash
# bump the version (no trailing newline)
printf '0.2.0' > resources/VERSION
git add resources/VERSION && git commit -m "Release 0.2.0"

# tag v0.2.0 and push the tag -> triggers the release workflow
lgx release
```

`lgx release` runs `scripts/tag.lg` (creates `v0.2.0` from `resources/VERSION`)
then `git push --tags`. Watch the run under the repo's Actions tab; when it
finishes there's a GitHub Release with four archives and an updated formula in
the tap. Then:

```bash
brew install <owner>/tap/<name>
# or, already tapped:
brew update && brew upgrade <name>
```

## README Installation section

Add this near the top of the README (adjust `<name>`/`<owner>`/`<repo>`):

````markdown
## Installation

### With [Homebrew](https://brew.sh)

Works on macOS and Linux:

```sh
brew install <owner>/tap/<name>
```

### With [mise](https://mise.jdx.dev)

```sh
mise use -g github:<owner>/<repo>@latest
```

Or pin a version in `.mise.toml`:

```toml
[tools]
"github:<owner>/<repo>" = "latest"
```

### Manual

Download the archive for your platform from the
[releases page](https://github.com/<owner>/<repo>/releases), extract it, and put
`<name>` on your `PATH`:

```sh
VERSION=0.1.0
OS=$(uname -s | tr '[:upper:]' '[:lower:]')   # linux | darwin
ARCH=$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')
curl -sSL -o <name>.tar.gz \
  "https://github.com/<owner>/<repo>/releases/download/v${VERSION}/<name>_${VERSION}_${OS}_${ARCH}.tar.gz"
tar -xzf <name>.tar.gz
mv <name> ~/.local/bin/
```
````
