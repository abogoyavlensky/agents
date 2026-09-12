# Egress control spike: transparent Squid

A candidate replacement for `sandbox/agent.yaml` plus `sandbox/egress/`: one
file, no per-tool proxy settings, allowlist edits applied live. Status:
config-checked on Linux, **not yet run on a Mac**. Everything in this directory
is throwaway until the spike passes.

## What it enforces

- `agent` has no sudo, except Lima's readiness probe and `agent-egress-reload`.
- nftables lets the agent uid (and its rootless-Docker subuid range) reach
  loopback only, and redirects its new TCP 80/443 connections into Squid.
- Squid splices TLS without decrypting it, only when the ClientHello SNI is on
  the allowlist and the destination IP is one that name resolves to. Plain HTTP
  needs an allowlisted host. Anything else on those ports gets an error.
- dnsmasq is the only resolver and forwards allowlisted names only; every other
  name is NXDOMAIN, so tools fail fast and DNS carries nothing out.

Tools and containers need no proxy configuration: they resolve names and connect
as usual, and the interception happens underneath them.

## Try it

```bash
mkdir -p ~/.config/lima-sandbox            # policy dir; may stay empty at first
limactl create --name egress-spike sandbox/spike/agent.yaml
limactl start egress-spike
```

Mount your projects with `--mount`, not `--mount-only`, which would drop the
policy mount:

```bash
limactl stop egress-spike
cd ~/Projects && limactl edit egress-spike --mount .:w --start
```

Run the acceptance tests from the Mac; they execute inside the VM as `agent`:

```bash
limactl shell egress-spike bash -s < sandbox/spike/verify.sh
```

Then use it for real for a day: log in to Claude Code and Codex, run a JVM
project, pull an image, run testcontainers.

## Allowlist

Create `~/.config/lima-sandbox/allowlist.txt` on the Mac. One hostname per line,
`#` comments, an entry covers itself and all subdomains. The built-in default
(see `allowlist.default` in `agent.yaml`) is used only until this file exists.
IP literals, `localhost` and `.local`/`.internal` names are rejected.

Apply an edit:

```bash
limactl shell egress-spike sudo agent-egress-reload
```

This restarts dnsmasq and Squid, so open tunnels to a removed host are cut too.
The agent may run it as well; it only re-reads root-owned or host-owned input.

Find out what was blocked:

```bash
limactl shell egress-spike journalctl -t dnsmasq -o cat -f | grep NXDOMAIN   # names
limactl shell egress-spike journalctl -t squid -o cat -f | grep -v TUNNEL/200 # connections
```

## Go / no-go

Go when all of these hold on the Mac:

1. `verify.sh` passes, including the container checks.
2. Claude Code, Codex and opencode log in and work; `mise` installs a runtime.
3. The 409 count `verify.sh` prints stays near zero over a day of use. A 409 is
   Squid refusing a connection because the destination IP was not one the name
   resolved to. Legitimate ones come from CDNs rotating addresses between the
   client's lookup and Squid's; if they are frequent, this design is a no-go and
   the fallback is the DNS-driven design (dnsmasq `nftset`, no Squid).
4. The policy mount is readable in the VM (`verify.sh` reports it).

## Known limits, accepted for the spike

- IPv4 only outside the VM. IPv6 loopback still works for local servers.
- Encrypted Client Hello is not inspected; a client using it is refused because
  the outer name is not allowlisted.
- Allowed hosts remain exfiltration channels (a gist is an upload). Token
  scoping is the control for that.
- `agent-egress-reload` interrupts open connections by design.

## Differences from `sandbox/agent.yaml`

No Homebrew (opencode via its installer, revdiff via `mise use -g ubi:...`), no
`admin` account, no `mitmproxy`, no proxy environment, no Maven or Docker proxy
files, no `agent-egress-verify` in the VM.
