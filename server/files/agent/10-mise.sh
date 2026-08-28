#!/usr/bin/env bash
# Runs as: sudo -u agent -H /usr/local/lib/agent-setup/10-mise.sh
# Installs mise if absent, then installs everything declared in config.toml.
set -euo pipefail

# Set PATH explicitly. Do NOT rely on `bash -lc`: Ubuntu's ~/.profile adds
# ~/.local/bin (so mise itself resolves) but never adds the mise shims dir,
# so every mise-MANAGED tool is missing - and `gh` silently resolves to
# /usr/bin/gh instead of the mise one. Verified on a live box.
export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$PATH"
export MISE_YES=1                      # never prompt on a TTY-less connection

if [ ! -x "$HOME/.local/bin/mise" ]; then
  curl -fsSL https://mise.run | sh
fi

mise trust "$HOME/.config/mise/config.toml"
mise install --yes
mise reshim
mise ls --current
