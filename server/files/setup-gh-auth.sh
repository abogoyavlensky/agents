#!/usr/bin/env bash
# Authenticate gh for the agent user WITHOUT the token touching argv,
# shell history, the screen, or any file on disk.
set -euo pipefail

[ "$(id -un)" = "agent" ] || { echo "ERROR: run this as the agent user (ssh agent@host)."; exit 1; }

export PATH="$HOME/.local/bin:$PATH"
eval "$("$HOME/.local/bin/mise" activate bash --shims)"

read -rsp "Paste GitHub token (input is hidden), then press Enter: " TOKEN; echo
[ -n "${TOKEN:-}" ] || { echo "ERROR: empty token."; exit 1; }

# piped on stdin, never as an argument -> not visible in /proc/*/cmdline
printf '%s' "$TOKEN" | gh auth login --hostname github.com --git-protocol https --with-token
unset TOKEN

gh auth setup-git                       # make git push/pull use gh's credentials
chmod 700 "$HOME/.config/gh" 2>/dev/null || true
chmod 600 "$HOME/.config/gh"/*.yml 2>/dev/null || true

echo
echo "=== gh auth status ==="
gh auth status
echo
echo "=== token identity + accessible repos ==="
gh api user --jq '"authenticated as: " + .login'
gh repo list --limit 10 --json nameWithOwner --jq '.[].nameWithOwner' || true
echo
echo "Done. Token stored 0600 in ~/.config/gh/hosts.yml"
