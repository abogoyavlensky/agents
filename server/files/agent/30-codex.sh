#!/usr/bin/env bash
# Runs as: sudo -u agent -H /usr/local/lib/agent-setup/30-codex.sh
set -euo pipefail
export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$PATH"

npm install -g @openai/codex

# REQUIRED. codex is an npm -g package inside the mise node install, so it is
# reachable only through a mise shim. Verified: shims/codex -> mise. Without
# a reshim the shim never appears and `codex` is on no PATH anywhere.
mise reshim

command -v codex
codex --version
