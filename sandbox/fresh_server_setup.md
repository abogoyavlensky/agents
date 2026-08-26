Agentic Development Server Setup

Sets up a Linux server so AI coding agents (Claude Code, Codex) can work on your repos without being able to damage the host or reach production secrets.

Model: root owns the machine; a non-privileged agent user owns the work. The boundary is a real Unix user, not a config flag.

Target: Ubuntu 22.04/24.04. Run every command as root unless marked otherwise.

---

Step 0 — Survey before you change anything

Don't skip this. On a repurposed box it's where the surprises are.

# What's actually running (NOT `docker ps -a` — that lists dead containers too)
docker ps
systemctl list-units --type=service --state=running | head -30

# Capacity
free -h; nproc; df -h /
journalctl --disk-usage          # commonly multiple GB on long-uptime boxes

# Who already exists, and who has power
awk -F: '$3>=1000 && $3<65534 {print $1, $3, $6}' /etc/passwd
getent group sudo docker
grep -rEv '^\s*(#|$)' /etc/sudoers /etc/sudoers.d/

# Secrets an unprivileged user might read
stat -c '%A %U' /root
find /home -name '.env' -o -name 'secrets*' 2>/dev/null | xargs -r stat -c '%A %U %n'

Anything world-readable under /home containing credentials → chmod 600 it now.

---

Step 1 — System baseline

apt update && apt install -y \
  git curl ca-certificates build-essential \
  tmux ripgrep jq unzip

---

Step 2 — Disk hygiene

systemd-journald defaults to consuming up to 10% of the filesystem. On a long-uptime box that's often several GB.

sed -i 's/^\[Journal\]/[Journal]\nSystemMaxUse=200M\nSystemMaxFileSize=50M/' /etc/systemd/journald.conf
journalctl --vacuum-size=200M
systemctl restart systemd-journald
journalctl --disk-usage      # verify

If Docker is present, reclaim dead containers — but check before pruning images:

docker container prune -f
docker images -f dangling=true -q | wc -l   # if 0, `image prune` frees nothing

Untagged (dangling) images are garbage. Tagged ones may be deployment rollback targets — don't delete those reflexively.

---

Step 3 — Swap

Agent workloads are bursty. Without swap, an overcommit is an instant OOM kill instead of a slowdown.

fallocate -l 4G /swapfile
chmod 600 /swapfile
mkswap /swapfile && swapon /swapfile
echo '/swapfile none swap sw 0 0' >> /etc/fstab

printf 'vm.swappiness=10\nvm.vfs_cache_pressure=50\n' > /etc/sysctl.d/99-agent-vm.conf
sysctl -p /etc/sysctl.d/99-agent-vm.conf
swapon --show

swappiness=10 keeps swap as a safety net rather than a routine destination.

---

Step 4 — Create the agent user

useradd -m -s /bin/bash -c "Agent development user" agent
chmod 700 /home/agent          # default is 0750; 0700 is stricter
install -d -m 755 -o agent -g agent /home/agent/repos

▎ ⚠️ Never add agent to the docker group
▎
▎ Docker group membership is root-equivalent — docker run -v /:/host and the boundary is iners, that's a request to a human, not a group membership. Same for sudo. Verify both areempty:
▎ getent group docker sudo

Give it SSH access with your existing keys:

install -d -m 700 -o agent -g agent /home/agent/.ssh
install -m 600 -o agent -g agent /root/.ssh/authorized_keys /home/agent/.ssh/authorized_keys

---

Step 5 — Resource limits

Caps a runaway build so it dies in its own cgroup instead of taking down the host.

AGENT_UID=$(id -u agent)
mkdir -p /etc/systemd/system/user-${AGENT_UID}.slice.d

cat > /etc/systemd/system/user-${AGENT_UID}.slice.d/50-agent-limits.conf <<'EOF'
[Slice]
MemoryHigh=2G          # soft: throttle + reclaim
MemoryMax=3G           # hard wall
MemorySwapMax=3G
CPUQuota=180%          # of (cores x 100%)
CPUWeight=50
IOWeight=50
TasksMax=4096
EOF

systemctl daemon-reload
loginctl enable-linger agent    # user manager + tmux survive logout

Sizing: MemoryHigh ≈ 55% of RAM, MemoryMax ≈ 75%, CPUQuota ≈ (cores × 100) − 20%.

▎ ⚠️ Limits apply to login sessions only
▎
▎ This is the non-obvious part. Processes land in user-<UID>.slice only via a real login
▎ ssh agent@host                # -> user-1001.slice   CAPPED
▎ sudo -u agent <cmd>           # -> user-0.slice      UNCAPPED
▎ Driving agents via sudo -u agent from a root shell silently gets you no memory protection. Use SSH (or machinectl shell agent@). Verify with cat /proc/self/cgroup.

---

Step 6 — Toolchain (as agent, not root)

ssh agent@<host>       # do this as the agent user

# mise: manages language runtimes + CLI tools, no sudo needed
curl -fsSL https://mise.run | sh

cat >> ~/.bashrc <<'EOF'

# --- agent dev env ---
export PATH="$HOME/.local/bin:$PATH"
eval "$($HOME/.local/bin/mise activate bash)"
EOF
source ~/.bashrc

mise use -g node@24 gh@latest

curl -fsSL https://claude.ai/install.sh | bash   # Claude Code
npm install -g @openai/codex                     # Codex

git config --global user.name  "your-name"
git config --global user.email "you@example.com"
git config --global init.defaultBranch main

Why mise instead of Homebrew: mise covers language runtimes plus most CLI tools (via its ubi/aqua backends), installs per-user with no sudo, and doesn't fight multi-user setups. Homebrew on Linux wants a
single-owner prefix and adds ~1 GB. Use apt as root for system libraries, mise as agent f

---

Step 7 — GitHub access via scoped token

7a. Create a fine-grained PAT

Settings → Developer settings → Personal access tokens → Fine-grained tokens

- Repository access: Only select repositories → just the agent's working set
  - "All repositories" also auto-includes every repo you create in the future — the scope grows without you deciding
- Expiration: finite (90 days)

┌─────────────────┬────────────────┬────────────────────────────────────────────────────────────────────┐
│   Permission    │     Level      │                                               Why                                               │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Contents        │ Read and write │ clone, commit, push                                                                             │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Metadata        │ Read-only      │ mandatory                                                                                       │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Pull requests   │ Read and write │ open/update PRs                                                                                 │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Issues          │ Read and write │ if the agent manages issues                                                                     │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Actions         │ Read-only      │ check CI results                                                                                │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Commit statuses │ Read-only      │ PR pass/fail                                                                                    │
├─────────────────┼────────────────┼────────────────────────────────────────────────────────────────────┤
│ Workflows       │ Read and write │ only if it edits .github/workflows/ — without it, any push touching a workflow file is rejected │
└─────────────────┴────────────────┴────────────────────────────────────────────────────────────────────┘

Grant no administration or delete permissions.

Prefer this over a classic PAT: classic repo scope grants every repo you own plus webhookings.

7b. Authenticate without leaking the token

Never paste a token into an agent chat session or a shell command — it lands in transcripc/*/cmdline.

Save as /home/agent/setup-gh-auth.sh (root-owned, chmod 755):

#!/usr/bin/env bash
set -euo pipefail
[ "$(id -un)" = "agent" ] || { echo "ERROR: run as agent"; exit 1; }

export PATH="$HOME/.local/bin:$PATH"
eval "$("$HOME/.local/bin/mise" activate bash --shims)"

read -rsp "Paste GitHub token (hidden), then Enter: " TOKEN; echo
[ -n "${TOKEN:-}" ] || { echo "ERROR: empty token."; exit 1; }

# piped on stdin, never in argv
printf '%s' "$TOKEN" | gh auth login --hostname github.com --git-protocol https --with-token
unset TOKEN

gh auth setup-git                       # make git push/pull use gh credentials
chmod 700 "$HOME/.config/gh"
chmod 600 "$HOME/.config/gh"/*.yml 2>/dev/null || true

gh auth status
gh repo list --limit 10

Run it as agent over SSH.

▎ gh auth status reports no scopes for fine-grained tokens — that's normal, not a failure. They don't advertise via the X-OAuth-Scopes header. Judge by the repo list instead.

To narrow scope later: edit the token's repository access on GitHub. The token value doesn't change, so no re-auth is needed.

---

Step 8 — Verify the boundary

Every one of these must be denied:

sudo -n true                                    # no sudo
cat /root/.ssh/authorized_keys                  # can't read root
docker ps                                       # no docker socket
find /home/agent -perm -4000 -o -perm -2000     # no setuid binaries

Confirm the agent's tools run unprivileged:

systemd-run --uid=agent --gid=agent --slice=user-$(id -u agent).slice --scope \
  bash -c 'id -un; echo euid=$EUID'             # -> agent, euid=1001

Confirm limits are live:

systemctl show user-$(id -u agent).slice -p MemoryMax -p CPUQuotaPerSecUSec

Confirm the token's ceiling — these must all be blocked:

gh api /repos/OWNER/REPO/actions/secrets    # secrets
gh api /repos/OWNER/REPO/keys               # deploy keys
gh api /repos/OWNER/REPO/hooks              # webhooks
gh api /user/emails                         # account

---

Operating rules

1. Never execute agent-owned code as root. The boundary holds in one direction only. sudoileges; root touching agent files does not. This hands over root:

cd /home/agent/repos/project && npm install    # postinstall runs as root ✗

Same for git hooks, mise tasks, Makefile targets, and source-ing anything under /home/ageand machine config — nothing else.

2. Agents inherit the agent user's reach. Anything running as agent can read ~/.config/ghe — it's how they work — and it's exactly why token scoping is the real control.

3. Reach agents over SSH, not sudo -u. Otherwise the memory caps silently don't apply.

4. One shared agent user is usually right. Agents collaborating on the same repos want thh git worktrees and separate tmux sessions, not separate users. Isolation from productionis what the user boundary buys you.

---

Sizing                                                                                                                                                                                                        
Minimum 2 vCPU / 4 GB / 40 GB; recommended 4 vCPU / 8 GB / 80 GB. One claude process is ~370 MB, so two concurrent agents plus a language server and a test suite do not fit comfortably in 4 GB. RAM matters most, CPU second, disk least. On Hetzner, CPU/RAM resizes are reversible — disk expansion