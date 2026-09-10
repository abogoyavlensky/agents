# Agent Sandbox using Lima VM

## Install Lima

```bash
brew install lima
```

## Create the agent sandbox

```bash
limactl create --name sandbox ./agent.yaml
```

## Mount projects dir

Mount the directory you want the VM to see and start the VM. You only
need to do this once; you can change the mount later.

```bash
cd ~/Projects   # or any directory you want to expose
limactl edit sandbox --mount-only .:w --start
```


## Share skills dir with agents

```bash
limactl stop sandbox
limactl edit sandbox
```

Then, if your skills directory is outside of the `Projects` directory you mounted above, 
add 

```bash
mounts:
...
- location: "/Users/andrew/Projects/agents/skills"
  mountPoint: "/home/agent.guest/.claude/skills"
  writable: true
- location: "/Users/andrew/Projects/agents/skills"
  mountPoint: "/home/agent.guest/.agents/skills"
  writable: true
- location: "/Users/andrew/Projects/agents/skills"
  mountPoint: "/home/agent.guest/.kiro/skills"
  writable: true
```

Or if you have your skills dir inside the `Projects` directory, you can just 
link it to the right place in the VM:

```bash
ln -s /workspace/Projects/agents/skills ~/.claude/skills
ln -s /workspace/Projects/agents/skills ~/.agents/skills
ln -s /workspace/Projects/agents/skills ~/.kiro/skills
```

## Set your git identity inside the VM

```shell
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## Aliases

```bash
lm() {
  LIMA_SHELLENV_BLOCK=* LIMA_SHELLENV_ALLOW=GH_TOKEN limactl shell --preserve-env $LIMA_DEFAULT_VM -- "$@"
}

# Open a shell in the VM with GH_TOKEN forwarded: `lmsh`
lmsh() {
  LIMA_SHELLENV_BLOCK=* LIMA_SHELLENV_ALLOW=GH_TOKEN limactl shell --preserve-env $LIMA_DEFAULT_VM
}

lmcc() {
  lm claude --dangerously-skip-permissions "$@"
}

lmcx() {
  lm codex --yolo "$@"
}

lmoc() {
  lm opencode "$@"
}

lmpi() {
  lm pi "$@"
}
```

## Egress control

The VM restricts what the agents can reach on the network. Three things are
enforced:

- **No sudo for `agent`.** The only command it keeps is
  `cat /mnt/lima-cidata/param.env`, which Lima's host agent runs on every start
  to check the guest is ready. Everything else is denied by
  `/etc/sudoers.d/zz-agent-egress`.
- **No direct network for `agent`.** nftables (`table inet agent-egress`) lets
  the agent uid, and its whole subordinate uid range, talk to loopback and
  nothing else. DNS is gone too: systemd-resolved is disabled, so the agent
  cannot resolve names at all.
- **An allowlisting proxy on 127.0.0.1:8080.** `agent-proxy.service` runs
  mitmproxy as its own user and allows only HTTPS on 443 and HTTP on 80 to
  hostnames on the allowlist. It does not intercept TLS; it compares the
  ClientHello's SNI with the host the client asked for and refuses the
  connection when they differ.

Tools pick the proxy up from `/etc/profile.d/agent-proxy.sh`, which every login
shell reads - and every way into the VM (`limactl shell`, ssh, `mode: user`
provisioning) is a login shell.

### Admin access

`agent` cannot fix any of this from inside. Use the separate `admin` account,
which has passwordless sudo:

```bash
lmadmin() {
  ssh -F ~/.lima/sandbox/ssh.config -o ControlPath=none -l admin lima-sandbox "$@"
}
```

`ControlPath=none` matters: Lima multiplexes ssh over a control socket, and
reusing it would land you back in the agent's session.

### Allowlist workflow

The allowlist lives at `/etc/agent-proxy/allowlist.txt` in the VM, seeded from
`sandbox/egress/allowlist.txt`. One host per line; an entry matches itself and
all its subdomains; `#` starts a comment. A missing or empty file denies
everything. Edits are picked up automatically when the file's mtime changes -
no restart needed.

It ships with a single `*` line, which allows every hostname while still
logging every decision. After a week of real use, build the real list from the
log and drop the `*`:

```bash
lmadmin sudo cat /var/log/agent-proxy/decisions.log |
  awk '$2 == "allow" && ($3 == "connect" || $3 == "http") { sub(/:[0-9]+$/, "", $4); print $4 }' |
  sort -u
```

Edit it in the VM with `lmadmin sudo nano /etc/agent-proxy/allowlist.txt`, or
push the repo's copy:

```bash
lmadmin sudo tee /etc/agent-proxy/allowlist.txt < sandbox/egress/allowlist.txt
```

Never point the proxy at a copy under the mounted `Projects` directory: this
repo lives there, so the agent could edit its own allowlist.

### Verifying

After creating or restarting the VM:

```bash
limactl shell sandbox agent-egress-verify
```

It runs ten checks - sudo, direct connections, DNS, the proxy's port and
IP-literal rules, the decision log, and SNI mismatch - and exits non-zero if
any fail.

### Changing the egress files

`mode: data` files are embedded into the instance config when the instance is
created, so edits to `sandbox/egress/*` only reach an existing VM through
`limactl delete` and `limactl create` again (or by editing the embedded copy
with `limactl edit sandbox`).

### Notes and limitations

- **JVM.** `JAVA_TOOL_OPTIONS` carries the proxy settings, so every JVM launch
  prints one `Picked up JAVA_TOOL_OPTIONS` line to stderr. Comment those two
  lines out of `/etc/profile.d/agent-proxy.sh` if it gets annoying - Maven and
  the Clojure CLI have their own settings in `~/.m2/settings.xml` anyway.
- **Docker.** Rootless dockerd reaches the proxy at `http://10.0.2.2:8080`
  (rootlesskit's view of the VM's loopback), configured in a user unit drop-in
  and `~/.docker/config.json`. Image pulls, builds and containers inherit it.
  Containers have no external DNS: anything inside a container that ignores the
  proxy has no network.
- **HTTPS and HTTP only.** ssh-based git, `ping`, raw DNS and every other
  protocol are unavailable to the agent. Use HTTPS remotes with the forwarded
  `GH_TOKEN`.
- **Allowed hosts are still exfiltration channels** - a gist on github.com is
  as good as any other upload. Token scoping and agent hooks are the controls
  for that, not this.
- **Tools that ignore the proxy variables fail fast** with connection refused
  until their own proxy setting is configured.
