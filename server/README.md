# Agentic Dev Server via spot

Provisions a fresh Ubuntu 24.04 server as an agentic-development box, using
[spot](https://spotctl.com) — a single Go binary, agentless, over SSH.

This automates Steps 1–12 of [`../sandbox/fresh_server_setup.md`](../sandbox/fresh_server_setup.md).
**Read that runbook first** — it explains *why* each step exists. This directory
is the executable form; the runbook is the reasoning.

Sized for **6 vCPU / 12 GB**. Connects as `root`.

## Install spot

```bash
mise use -g "github:umputun/spot"
```

(`github:`, not `ubi:` — the ubi backend is deprecated upstream.)

## Configure the target

```bash
cp inventory.yml.example inventory.yml   # gitignored; holds the real address
$EDITOR inventory.yml
```

## Run

```bash
spot -p spot.yml -i inventory.yml -t agentbox -n survey   # read-only, review first
spot -p spot.yml -i inventory.yml -t agentbox             # tasks 1-11
```

Then the two steps that **must** be done by a human (see below):

```bash
ssh agent@<host> ./setup-gh-auth.sh                       # MANUAL 1 - GitHub token
spot -p spot.yml -i inventory.yml -t agentbox -n skills
ssh agent@<host>                                          # MANUAL 2 - permission rule
spot -p spot.yml -i inventory.yml -t agentbox -n verify-github
```

## Tasks

| Task | Auto | What |
|---|---|---|
| `survey` | no | Read-only audit: capacity, users, privileged groups, secrets in argv |
| `packages` | yes | apt baseline; refuses to run on anything but 24.04 |
| `docker` | yes | Engine from the official repo; asserts `agent` ∉ `docker` |
| `journald` | yes | 200 MB cap via drop-in |
| `swap` | yes | 8 GB swapfile, `vm.swappiness=10` |
| `agent-user` | yes | `agent` at **uid 1001**, home 0700, SSH keys, no sudo/docker |
| `limits` | yes | cgroup slice: MemoryHigh 6500M / Max 9G / CPUQuota 580% |
| `agent-files` | yes | `.bashrc.agent`, `.bash_profile`, mise config, gh helper, setup scripts |
| `toolchain` | yes | mise → node/gh/revdiff/opencode/spot/skl, Claude Code, Codex |
| `git-config` | yes | Identity, `insteadOf` rewrites, git aliases |
| `verify` | yes | Boundary + limits + toolchain + idempotency counters |
| `skills` | no | Clone agents repo, link skills — *needs gh auth first* |
| `bwrap-apparmor` | no | Exempt bwrap from 24.04 userns hardening — *see below* |
| `verify-github` | no | Assert the token can't reach secrets/keys/hooks — *needs gh auth* |

Run a single task with `-n <task>`.

## What is deliberately NOT automated

**1. The GitHub token.** `setup-gh-auth.sh` uses an interactive hidden `read -rsp`
and pipes the token on stdin, never through argv. A token must never reach this
playbook, this repo, or a process list. The playbook installs the script
**root-owned `0755`** — the agent may execute it but must not be able to edit the
thing a human types a token into.

**2. The Claude Code permission rule** (`Bash(codex exec:*)` in
`~/.claude/settings.json`). This is a two-line `jq` and spot would run it fine.
It stays manual as *policy, not capability*: Claude Code's classifier refuses to
let an agent write its own permission rules, through both Bash and Edit, and
being root does not help — it gates the action, not the file. An agent that can
widen its own permissions can grant itself anything. **Do not "helpfully"
automate this.**

**3. `bwrap-apparmor` is opt-in for a reason.** It only pays off if you intend to
run codex *sandboxed here*. If your other environments (the Lima VM) cannot
sandbox, the skills must keep `--dangerously-bypass-approvals-and-sandbox` to
work there — and then codex never invokes bwrap at all, so this buys nothing.

## Proving idempotency

spot is **not declarative**. Its docs are explicit: tasks are "a direct list of
straightforward commands," and it does not enforce idempotency. Every guard in
`spot.yml` is hand-written. So prove it empirically:

```bash
spot -p spot.yml -i inventory.yml -t agentbox                     # run 1
ssh root@<host> 'md5sum /etc/fstab /etc/sysctl.d/99-agent-vm.conf \
  /etc/systemd/journald.conf.d/99-agent.conf \
  /etc/systemd/system/user-1001.slice.d/50-agent-limits.conf \
  /home/agent/.bashrc /home/agent/.bash_profile /home/agent/.bashrc.agent \
  /home/agent/.gitconfig /home/agent/.config/mise/config.toml' > /tmp/run1.md5

spot -p spot.yml -i inventory.yml -t agentbox                     # run 2
ssh root@<host> 'md5sum ...same list...' > /tmp/run2.md5
diff /tmp/run1.md5 /tmp/run2.md5                                  # MUST be empty

spot -p spot.yml -i inventory.yml -t agentbox -n verify
```

`.gitconfig` is the one to watch — `git config --add insteadOf` is the only
command here that duplicates silently on every re-run, and the count is the only
visible symptom. `verify` asserts it is exactly 2.

**Definition of done: run the playbook twice, then `-n verify` green.**

> `--dry` is not an idempotency check. It prints commands without running them,
> does not evaluate `cond:`, and so over-reports what would happen. It also
> still opens an SSH connection — it fails outright against an unreachable host.

## Gotchas

**`bash -lc` does not reach mise-managed tools.** Ubuntu's `~/.profile` adds
`~/.local/bin`, so `bash -lc mise` works — but the mise *shims* dir is never on
PATH outside interactive shells, so `bash -lc node` fails and `bash -lc gh`
silently resolves to `/usr/bin/gh` instead of the mise one. Every script in
`files/agent/` therefore exports PATH explicitly as its first line. Do not
"simplify" that away.

**`mise reshim` after any `npm install -g`.** codex is an npm global inside the
mise node install, reachable only through a shim (`shims/codex -> mise`). No
reshim, no shim, and `codex` is on no PATH anywhere.

**`copy` cannot set owner or mode** (only `chmod+x`). Every copy landing under
`/home/agent` is followed by an explicit `chown`. The `line` command has the
same problem — it runs as root and can flip `.bashrc` to `root:root`, which
silently breaks the agent's shell, so that too is followed by a `chown`.

**`cond:` works on `script` and `echo` only** — not on `copy` or `line`.

**`NEEDRESTART_MODE=a` on every apt call.** Ubuntu 24.04's needrestart opens a
whiptail service-restart prompt that hangs a TTY-less run indefinitely.

**uid 1001 is pinned.** The slice drop-in path `user-1001.slice.d/` is
uid-derived and `copy` has no templating. The `agent-user` task asserts the uid
and fails loudly rather than writing limits to the wrong slice. A
`user-.slice.d/` template would avoid this but applies to *every* user slice,
including root's.

## Files

```
spot.yml                   playbook (a table of contents; logic lives in files/)
inventory.yml.example      copy to inventory.yml (gitignored)
files/
  50-agent-limits.conf     cgroup caps        -> /etc/systemd/system/user-1001.slice.d/
  99-agent-vm.conf         swappiness         -> /etc/sysctl.d/
  99-agent-journald.conf   journal cap        -> /etc/systemd/journald.conf.d/
  apparmor-bwrap           userns exemption   -> /etc/apparmor.d/bwrap
  bashrc.agent             managed shell env  -> /home/agent/ (agent:agent)
  bash_profile             login PATH         -> /home/agent/ (agent:agent)
  mise-config.toml         declared tools     -> /home/agent/.config/mise/ (agent:agent)
  setup-gh-auth.sh         token helper       -> /home/agent/ (root:root 0755)
  agent/*.sh               agent-side setup   -> /usr/local/lib/agent-setup/ (root:root)
```

`files/agent/*.sh` are root-owned and live outside `/home/agent` on purpose, and
are only ever invoked as `sudo -u agent -H <script>`. Root never *executes*
agent-writable code — it drops privileges to run root-owned code. That is
Operating rule 1 of the runbook, applied to the provisioner itself.
