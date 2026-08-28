#!/usr/bin/env bash
# Runs as: sudo -u agent -H /usr/local/lib/agent-setup/20-claude.sh
set -euo pipefail
export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$PATH"

curl -fsSL https://claude.ai/install.sh | bash

# The installer warns "~/.local/bin is not in your PATH" when it probes a
# non-interactive shell. Harmless here - bashrc.agent and bash_profile both
# add it. Verify directly rather than trusting the installer's own message.
"$HOME/.local/bin/claude" --version
