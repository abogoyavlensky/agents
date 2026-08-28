#!/usr/bin/env bash
# Runs as: sudo -u agent -H /usr/local/lib/agent-setup/40-git.sh
# Git identity, SSH->HTTPS rewrites, and agent-usable git aliases.
set -euo pipefail
export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$PATH"

git config --global user.name  "Andrey Bogoyavlenskiy"
git config --global user.email "abogoyavlensky@gmail.com"
git config --global init.defaultBranch main

# Agents copy SSH-style URLs off GitHub's web UI. Without these they fail with
# "Permission denied (publickey)" - the agent user has no SSH key, by design
# (an account-level GitHub SSH key cannot be repo-scoped and never expires).
#
# insteadOf is MULTI-VALUED: plain `git config` REPLACES, `--add` appends.
# Guard on the count, because --add duplicates silently on every re-run and
# the count is the only visible symptom.
if [ "$(git config --global --get-all url."https://github.com/".insteadOf | wc -l)" -ne 2 ]; then
  git config --global --unset-all url."https://github.com/".insteadOf 2>/dev/null || true
  git config --global --add url."https://github.com/".insteadOf "git@github.com:"
  git config --global --add url."https://github.com/".insteadOf "ssh://git@github.com/"
fi

# Real git aliases, unlike the shell aliases in bashrc.agent these DO work in
# the non-interactive shells agents actually use.
git config --global alias.s  status
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.lg "log --graph --pretty=format:'%C(bold)%h%Creset%C(magenta)%d%Creset %s %C(yellow)<%an> %C(cyan)(%cr)%Creset' --abbrev-commit --date=relative"

git config --global --get-all url."https://github.com/".insteadOf
