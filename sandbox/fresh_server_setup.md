# Agentic Development Server Setup

Sets up a Linux server so AI coding agents (Claude Code, Codex) can work on your
repos without being able to damage the host or reach production secrets.

**Model:** `root` owns the machine; a non-privileged `agent` user owns the work.
The boundary is a real Unix user, not a config flag.

**Target:** Ubuntu 22.04/24.04. Run every command as `root` unless marked
`[as agent]`.

---

## Step 0 — Survey before you change anything

Don't skip this. On a repurposed box it's where the surprises are.

```bash
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

# Secrets already exposed in process arguments (/proc/*/cmdline is world-readable)
ps -eo user,args | grep -iE 'token|bearer|api[-_]?key|password' | grep -v grep
```

Anything world-readable under `/home` containing credentials -> `chmod 600` it now.

---

## Step 1 — System baseline

```bash
apt update && apt install -y \
  git curl wget ca-certificates build-essential pkg-config \
  tmux ripgrep jq fzf unzip zip bzip2 file procps \
  openssh-client gnupg gpg-agent iproute2 \
  libssl-dev zlib1g-dev
```

---

## Step 2 — Disk hygiene

`systemd-journald` defaults to consuming up to 10% of the filesystem. On a
long-uptime box that's often several GB.

```bash
sed -i 's/^\[Journal\]/[Journal]\nSystemMaxUse=200M\nSystemMaxFileSize=50M/' /etc/systemd/journald.conf
journalctl --vacuum-size=200M
systemctl restart systemd-journald
journalctl --disk-usage      # verify: should now report ~200M, not GB
```

The `SystemMaxUse` cap is the part that matters — vacuuming alone lets it regrow.

If Docker is present, reclaim dead containers — but **check before pruning images**:

```bash
docker container prune -f
docker images -f dangling=true -q | wc -l   # if 0, `image prune` frees nothing
```

Untagged (`dangling`) images are garbage. Tagged ones may be deployment rollback
targets — don't delete those reflexively. `docker system df` shows what is
actually reclaimable.

---

## Step 3 — Swap

Agent workloads are bursty. Without swap, an overcommit is an instant OOM kill
instead of a slowdown.

```bash
fallocate -l 4G /swapfile
chmod 600 /swapfile
mkswap /swapfile && swapon /swapfile
echo '/swapfile none swap sw 0 0' >> /etc/fstab

printf 'vm.swappiness=10\nvm.vfs_cache_pressure=50\n' > /etc/sysctl.d/99-agent-vm.conf
sysctl -p /etc/sysctl.d/99-agent-vm.conf
swapon --show
```

`swappiness=10` keeps swap as a safety net rather than a routine destination.

---

## Step 4 — Create the agent user

```bash
useradd -m -s /bin/bash -c "Agent development user" agent
chmod 700 /home/agent          # Ubuntu HOME_MODE default is 0750; 0700 is stricter
install -d -m 755 -o agent -g agent /home/agent/Projects
```

> **WARNING: Never add `agent` to the `docker` group.**
> Docker group membership is **root-equivalent** — `docker run -v /:/host` and the
> boundary is gone. If an agent needs containers, that's a request to a human, not
> a group membership. Same for `sudo`. Verify both are empty:
> ```bash
> getent group docker sudo
> ```

Give it SSH access with your existing keys:

```bash
install -d -m 700 -o agent -g agent /home/agent/.ssh
install -m 600 -o agent -g agent /root/.ssh/authorized_keys /home/agent/.ssh/authorized_keys
```

---

## Step 5 — Resource limits

Caps a runaway build so it dies in its own cgroup instead of taking down the host.

```bash
AGENT_UID=$(id -u agent)
mkdir -p /etc/systemd/system/user-${AGENT_UID}.slice.d

cat > /etc/systemd/system/user-${AGENT_UID}.slice.d/50-agent-limits.conf <<'LIMITS'
[Slice]
MemoryHigh=2G          # soft: throttle + reclaim
MemoryMax=3G           # hard wall
MemorySwapMax=3G
CPUQuota=180%          # of (cores x 100%)
CPUWeight=50
IOWeight=50
TasksMax=4096
LIMITS

systemctl daemon-reload
loginctl enable-linger agent    # user manager + tmux survive logout
```

Sizing: `MemoryHigh` ~ 55% of RAM, `MemoryMax` ~ 75%,
`CPUQuota` ~ `(cores x 100) - 20`%.

> **WARNING: Limits apply to login sessions only.**
> This is the non-obvious part. Processes land in `user-<UID>.slice` **only** via a
> real login session:
> ```bash
> ssh agent@host                # -> user-1001.slice   CAPPED
> sudo -u agent <cmd>           # -> user-0.slice      UNCAPPED
> ```
> Driving agents via `sudo -u agent` from a root shell silently gets you **no**
> memory protection. Use SSH (or `machinectl shell agent@`). Verify with
> `cat /proc/self/cgroup`.

---

## Step 6 — Toolchain `[as agent]`

```bash
ssh agent@<host>
```

```bash
# mise: manages language runtimes + CLI tools, no sudo needed
curl -fsSL https://mise.run | sh

cat >> ~/.bashrc <<'ENVBLOCK'

# --- agent dev env ---
export PATH="$HOME/.local/bin:$PATH"
eval "$($HOME/.local/bin/mise activate bash)"
export GH_PROMPT_DISABLED=1     # never block an agent on an interactive gh prompt
ENVBLOCK
source ~/.bashrc

mise use -g node@24 gh@latest

# tools that aren't in mise's registry: use the ubi backend (GitHub releases)
mise use -g "ubi:umputun/revdiff"

curl -fsSL https://claude.ai/install.sh | bash   # Claude Code
npm install -g @openai/codex                     # Codex
```

**Why mise instead of Homebrew:** mise covers language runtimes plus most CLI
tools, installs per-user with no sudo, and doesn't fight multi-user setups.
Homebrew on Linux wants a single-owner prefix and adds ~1 GB. Anything missing
from mise's registry is usually one `ubi:owner/repo` away. Use `apt` as root for
system libraries, mise as agent for everything else.

---

## Step 7 — Shell environment and git aliases `[as agent]`

```bash
git config --global user.name  "Your Name"        # match your real commit identity
git config --global user.email "you@example.com"
git config --global init.defaultBranch main

cat >> ~/.bashrc <<'ALIASES'

# --- git aliases ---
alias gs="git status"
alias gpo='git push origin HEAD'
alias gg='git log --graph --pretty=format:'\''%C(bold)%h%Creset%C(magenta)%d%Creset %s %C(yellow)<%an> %C(cyan)(%cr)%Creset'\'' --abbrev-commit --date=relative'
alias gcam='git commit -v -am'
alias gcmp='git checkout master && git pull origin master'
alias gcb='git checkout -b $1'
alias rd='revdiff "$@"'
ALIASES
```

Verify with `alias gs gpo gg gcam gcmp gcb rd` and by running `gg -3` in a repo.

> **Gotcha: aliases do not reach agents.** Bash disables alias expansion in
> non-interactive shells, and Ubuntu's stock `.bashrc` returns early for them. So
> `bash -c 'gs'` — which is how an agent runs commands — gets *nothing*. These are
> for humans at a prompt. If you want a shortcut an agent can use, make it a real
> git alias in `~/.gitconfig` (`git config --global alias.s status`), which works
> in every context.

> **Gotcha: `$1` and `"$@"` do nothing in an alias.** `gcb` and `rd` above work
> only by accident — alias expansion is textual, so the argument is appended after
> the (empty) `$1`. Use `alias gcb='git checkout -b'`, or a shell function if you
> genuinely need positional parameters.

Note `gcmp` hard-codes `master`. It fails on `main`-default repos.

---

## Step 8 — GitHub access via scoped token

### 8a. Create a **fine-grained** PAT

*Settings -> Developer settings -> Personal access tokens -> Fine-grained tokens*

- **Repository access:** *Only select repositories* -> just the agent's working set
  - "All repositories" also auto-includes every repo you create **in the future** —
    the scope grows without you deciding
- **Expiration:** finite (90 days)

| Permission | Level | Why |
|---|---|---|
| Contents | Read and write | clone, commit, push |
| Metadata | Read-only | mandatory |
| Pull requests | Read and write | open/update PRs |
| Issues | Read and write | if the agent manages issues |
| Actions | Read-only | check CI results |
| Commit statuses | Read-only | PR pass/fail |
| Workflows | Read and write | **only** if it edits `.github/workflows/` — without it, *any* push touching a workflow file is rejected |

Grant no administration or delete permissions.

Prefer this over a classic PAT: classic `repo` scope grants every repo you own
*plus* webhooks, deploy keys, and repo settings.

### 8b. Authenticate without leaking the token

Never paste a token into an agent chat session or a shell command — it lands in
transcripts, `~/.bash_history`, and `/proc/*/cmdline`.

Save as `/home/agent/setup-gh-auth.sh` (root-owned, `chmod 755`):

```bash
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
```

Run it as `agent` over SSH.

> `gh auth status` reports **no scopes** for fine-grained tokens — that's normal,
> not a failure. They don't advertise via the `X-OAuth-Scopes` header. Judge by the
> repo list instead.

To narrow scope later: edit the token's repository access on GitHub. The token
value doesn't change, so **no re-auth is needed**.

### 8c. Make git "just work" for agents `[as agent]`

Agents routinely copy SSH-style URLs off GitHub's web UI. Without a rewrite those
fail with `Permission denied (publickey)`, because the agent user has no SSH key.

```bash
git config --global --add url."https://github.com/".insteadOf "git@github.com:"
git config --global --add url."https://github.com/".insteadOf "ssh://git@github.com/"

git config --global --get-all url."https://github.com/".insteadOf   # expect 2 lines
```

Note `--add`: `insteadOf` is multi-valued, so a plain `git config` **replaces** the
previous value instead of appending.

After this, all three URL forms clone and push identically:
`git@github.com:owner/repo.git`, `ssh://git@github.com/owner/repo.git`,
`https://github.com/owner/repo.git`.

**Don't add an SSH key instead.** A GitHub account-level SSH key cannot be scoped
to specific repos and never expires — it would undo the fine-grained token
scoping and can only be revoked wholesale. If you need SSH for one repo, use a
per-repo *deploy key* (which can be read-only).

> **Gotcha:** `gh` prints nothing on empty results when its output is not a TTY.
> `gh pr list` on a repo with no PRs returns 0 bytes and exit code 0 — not the
> "no open pull requests" message you see interactively. Check the exit code or
> use `--json` and parse `[]`. Silence is not failure.

---

## Step 9 — Shared skills `[as agent]`

Keep skills in one git repo and link it into both agent CLIs:

```bash
git clone https://github.com/OWNER/agents.git ~/Projects/agents
mkdir -p ~/.claude ~/.agents
ln -sfn ~/Projects/agents/skills ~/.claude/skills
ln -sfn ~/Projects/agents/skills ~/.agents/skills

ls ~/.claude/skills | wc -l                        # sanity
ls -d ~/.claude/skills/*/SKILL.md 2>/dev/null | wc -l   # how many are valid skills
```

`ln -sfn` is idempotent — without `-n` a re-run nests the link *inside* the
existing directory.

Trade-off: a directory-level link means you can't mix in a machine-local skill —
anything dropped in lands in the repo working tree. Link each skill individually
if you need that.

---

## Step 10 — Codex sandbox on Ubuntu 24.04

Ubuntu 24.04 ships `kernel.apparmor_restrict_unprivileged_userns=1`, which breaks
codex's bundled `bwrap`:

```
bwrap: setting up uid map: Permission denied
bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted
```

Diagnose:

```bash
sysctl kernel.apparmor_restrict_unprivileged_userns   # 1 = restricted
head -8 /etc/apparmor.d/unprivileged_userns           # first rule: audit deny capability,
```

Note `kernel.unprivileged_userns_clone` is usually still `1` — creating the
namespace is allowed. It's the AppArmor profile stripping capabilities *inside*
it, so **only an AppArmor change fixes this**; no other sysctl will.

**Scoped fix (preferred)** — keep the hardening, exempt one binary:

```bash
apt install -y bubblewrap          # gives a stable /usr/bin/bwrap path
tee /etc/apparmor.d/bwrap >/dev/null <<'PROFILE'
abi <abi/4.0>,
include <tunables/global>
profile bwrap /usr/bin/bwrap flags=(unconfined) {
  userns,
  include if exists <local/bwrap>
}
PROFILE
apparmor_parser -r /etc/apparmor.d/bwrap
bwrap --ro-bind / / --unshare-all /bin/true && echo OK
```

Pin the profile to `/usr/bin/bwrap`, never codex's vendored copy — that path
contains the node and codex versions and moves on every upgrade.

The blunt alternative is `kernel.apparmor_restrict_unprivileged_userns=0`, which
disables the hardening machine-wide. Avoid it on any box that also runs CI or
untrusted code.

> **Decide this before editing skills.** If your *other* environments (a Lima VM,
> a container) can't run the sandbox either, then your skills must keep
> `--dangerously-bypass-approvals-and-sandbox` to work there — and swapping in
> `--sandbox read-only` will break them. In that case skip this step entirely and
> use Step 11 instead. Fixing AppArmor here only pays off if you actually intend
> to run codex sandboxed.

---

## Step 11 — Claude Code permission rules `[as agent]`

Skills that invoke `codex exec` with the bypass flag get denied by the auto-mode
classifier. Allow the command explicitly in `~/.claude/settings.json`:

```json
{
  "permissions": {
    "defaultMode": "auto",
    "allow": ["Bash(codex exec:*)"]
  }
}
```

That one rule covers `codex exec`, `codex exec review`, and `codex exec resume`.
Allow rules bypass the classifier in auto mode (`autoMode.classifyAllShell`
defaults to false).

> **A human must do this.** Claude Code will not write its own permission rules —
> the classifier denies it through both Bash and Edit, and being root does not
> help, because it gates the *action*, not the file. This is deliberate: an agent
> that can widen its own permissions can grant itself anything. Use `/permissions`
> in the session, or edit the file yourself:
> ```bash
> jq '.permissions.allow += ["Bash(codex exec:*)"]' ~/.claude/settings.json > /tmp/s.json \
>   && mv /tmp/s.json ~/.claude/settings.json
> ```
> (`+=` on a missing key works — jq treats `null + [x]` as `[x]`.)

A running session may not reload immediately. Open `/permissions` once, or
restart the session.

---

## Step 12 — Verify the boundary `[as agent]`

Every one of these must be **denied**:

```bash
sudo -n true                                    # no sudo
cat /root/.ssh/authorized_keys                  # can't read root
docker ps                                       # no docker socket
find /home/agent -perm -4000 -o -perm -2000     # no setuid binaries
```

Confirm the agent's tools run unprivileged:

```bash
systemd-run --uid=agent --gid=agent --slice=user-$(id -u agent).slice --scope \
  bash -c 'id -un; echo euid=$EUID'             # -> agent, euid=1001
```

Confirm limits are live *and actually enforcing under load*:

```bash
systemctl show user-$(id -u agent).slice \
  -p MemoryCurrent -p MemoryHigh -p MemoryMax -p CPUQuotaPerSecUSec
```

`MemoryCurrent` climbing toward `MemoryHigh` while agents run is the proof the
slice is real. If it reads 0 while work is happening, your session isn't in the
slice — see the Step 5 warning.

Confirm the token's ceiling — these must all be **blocked**:

```bash
gh api /repos/OWNER/REPO/actions/secrets    # secrets
gh api /repos/OWNER/REPO/keys               # deploy keys
gh api /repos/OWNER/REPO/hooks              # webhooks
gh api /user/emails                         # account
```

End-to-end git check (clone, branch, push, delete):

```bash
git clone git@github.com:OWNER/SCRATCH.git /tmp/t && cd /tmp/t
git push -u origin HEAD:refs/heads/verify-tmp
git push origin --delete verify-tmp
```

---

## Operating rules

**1. Never execute agent-owned code as root.** The boundary holds in one direction
only. `sudo -u agent ...` safely *drops* privileges; root touching agent files does
not. This hands over root:

```bash
cd /home/agent/Projects/project && npm install    # postinstall runs as root  <-- BAD
```

Same for git hooks, `mise` tasks, `Makefile` targets, and `source`-ing anything
under `/home/agent`. Root is for `apt`, Docker, and machine config — nothing else.

**2. Agents inherit the agent user's reach.** Anything running as `agent` can read
`~/.config/gh/hosts.yml`. That's unavoidable — it's how they work — and it's
exactly why token scoping is the real control.

**3. Keep secrets out of `argv`.** `/proc/<pid>/cmdline` is world-readable, so any
local user — including a CI runner account — can read tokens passed as command
arguments. This bites in non-obvious places: an MCP server configured with
`--mcp-config '{"headers":{"Authorization":"Bearer ..."}}'` exposes that token to
every user on the box. Prefer a config file or an environment variable. Audit with:

```bash
ps -eo user,args | grep -iE 'token|bearer|api[-_]?key' | grep -v grep
```

**4. Reach agents over SSH, not `sudo -u`.** Otherwise the memory caps silently
don't apply.

**5. One shared `agent` user is usually right.** Agents collaborating on the same
repos want the same home. Isolate them with git worktrees and separate tmux
sessions, not separate users. Isolation *from production* is what the user
boundary buys you.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `Permission denied (publickey)` on clone | SSH-style URL, no SSH key | Step 8c URL rewrite |
| `bwrap: setting up uid map: Permission denied` | Ubuntu 24.04 userns hardening | Step 10 |
| Codex skill denied in auto mode | Bypass flag hits the classifier | Step 11 allow rule |
| `gh pr list` returns nothing | Not a TTY; empty result prints nothing | Check exit code / `--json` |
| Aliases missing in agent-run commands | Non-interactive bash skips aliases | Use git aliases instead |
| Agent OOMs but host survives | Slice cap working as designed | Raise `MemoryMax`, or add RAM |
| Host OOMs instead of the agent | Session not in the slice | Use SSH, not `sudo -u` |
| Disk fills again after weeks | journald regrew | Confirm `SystemMaxUse` is set |

---

## Sizing

Minimum 2 vCPU / 4 GB / 40 GB; **recommended 4 vCPU / 8 GB / 80 GB**.

Measured on a live box: one `claude` process is ~370 MB, and an agent session
doing ordinary work (Claude + a node runtime + a tunnel) sits around **1.8 GB** —
against a 2 GB soft cap. Two concurrent agents plus a language server and a test
suite do not fit comfortably in 4 GB.

RAM matters most, CPU second, disk least. Caches are the disk hog: `~/.cache` and
`~/.npm` reach several GB and are entirely regenerable. On Hetzner, CPU/RAM
resizes are reversible — **disk expansion is one-way**.
