# Sandbox without sudo for the agent

`agent.yaml` here is the plain sandbox (same tools as `../agent.yaml`) with one
change: `agent` has no sudo, and a separate `admin` account has it. No proxy,
no firewall, no DNS changes.

## Setup

```bash
limactl create --name sandbox sandbox/nosudo/agent.yaml
cd ~/Projects && limactl edit sandbox --mount-only .:w --start
```

Add to your shell on the Mac:

```bash
lmadmin() {
  ssh -F ~/.lima/sandbox/ssh.config -o ControlPath=none -l admin lima-sandbox "$@"
}
```

`admin` uses the same ssh key Lima generated for the VM, so nothing else to set
up. `ControlPath=none` matters: without it ssh reuses Lima's multiplexed
connection and lands you in the agent's session.

## How it works in practice

- **You and the agent are the same user.** `limactl shell`, `lmcc`, `lmcx` all
  run as `agent`. Editing, git, mise, Homebrew, npm, pip, rootless Docker: all
  user-level, all unchanged. Agents never needed sudo for normal development.
- **Root work goes through `lmadmin`.** `lmadmin sudo apt-get install -y libfoo-dev`
  from the Mac, or `lmadmin` for a root-capable shell. Expect this rarely: build
  dependencies missing from the package list in `agent.yaml`, or debugging a
  system service. Add anything recurring to the package list and recreate.
- **An agent that hits sudo fails fast.** `sudo` prints a permission error; the
  agent reports it and you decide. Tell your agent, or put it in `AGENTS.md`,
  that the VM has no sudo and system packages are the human's job.
- **What it protects.** The VM's own plumbing: no kernel modules, no systemd
  units, no persistence across reboots, no reading Lima's root-only data. Any
  later control (firewall, proxy, audit log) has a real boundary to stand on.
- **What it does not protect.** Mounted directories, forwarded tokens, and the
  network are all reachable as `agent`. Mount one project rather than all of
  `~/Projects`, and scope `GH_TOKEN`, if those are the worry.

## Verify

```bash
limactl shell sandbox sudo -n true                        # must fail
limactl shell sandbox sudo -n cat /mnt/lima-cidata/param.env >/dev/null && echo ok  # Lima's probe
lmadmin sudo -n true && echo admin-ok
```
