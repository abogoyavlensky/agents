# gizmo

A tiny CLI that does the thing across all your repos.

## Installation

### With [Homebrew](https://brew.sh)

Works on macOS and Linux:

```sh
brew install acme/tap/gizmo
```

### With [mise](https://mise.jdx.dev)

```sh
mise use -g github:acme/gizmo@latest
```

Or pin a version in `.mise.toml`:

```toml
[tools]
"github:acme/gizmo" = "latest"
```

### Manual

Download the archive for your platform from the
[releases page](https://github.com/acme/gizmo/releases), extract it, and put
`gizmo` on your `PATH`:

```sh
VERSION=0.1.0
OS=$(uname -s | tr '[:upper:]' '[:lower:]')   # linux | darwin
ARCH=$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')
curl -sSL -o gizmo.tar.gz \
  "https://github.com/acme/gizmo/releases/download/v${VERSION}/gizmo_${VERSION}_${OS}_${ARCH}.tar.gz"
tar -xzf gizmo.tar.gz
mv gizmo ~/.local/bin/
```

## Development

```bash
lgx run -- run
lgx test
lgx fmt
lgx lint
```

Build a binary:

```bash
lgx build
bin/gizmo --version
```
