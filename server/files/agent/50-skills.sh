#!/usr/bin/env bash
# Runs as: sudo -u agent -H /usr/local/lib/agent-setup/50-skills.sh
# MANUAL TASK - needs gh auth first (spot -n skills).
set -euo pipefail
export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$PATH"

REPO="${AGENTS_REPO:-https://github.com/abogoyavlensky/agents.git}"
DEST="$HOME/Projects/agents"

if [ -d "$DEST/.git" ]; then
  git -C "$DEST" pull --ff-only
else
  mkdir -p "$HOME/Projects"
  git clone "$REPO" "$DEST"
fi

mkdir -p "$HOME/.claude" "$HOME/.agents"
# -n matters: without it a re-run nests the link INSIDE the existing dir.
ln -sfn "$DEST/skills" "$HOME/.claude/skills"
ln -sfn "$DEST/skills" "$HOME/.agents/skills"

echo "skills linked: $(ls "$HOME/.claude/skills" | wc -l) entries, $(ls -d "$HOME"/.claude/skills/*/SKILL.md 2>/dev/null | wc -l) with SKILL.md"
