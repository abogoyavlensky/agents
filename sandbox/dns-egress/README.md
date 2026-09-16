# Sandbox with a DNS-derived egress allowlist

`agent.yaml` here layers network control on `../nosudo/agent.yaml` (Lima's
`base:` merges it in; same tools, same `admin` account). Not yet run on a Mac.

How it works: dnsmasq is the VM's only resolver and answers only for names on
the allowlist; while answering, it adds the resolved addresses to an nftables
set, and the agent may open TCP 80/443 to those addresses only. Tools inside
the VM see nothing special: they resolve and connect as usual, and an unlisted
host fails at once with "could not resolve host". Containers included.

## Setup

```bash
limactl create --name sandbox sandbox/dns-egress/agent.yaml
cd ~/Projects && limactl edit sandbox --mount-only .:w --start
```

`lmadmin` from `../nosudo/README.md` is how you reach root.

## Allowlist

`/etc/agent-egress/allowlist.txt` in the VM. One hostname per line, `#`
comments, an entry covers itself and all subdomains. IP literals, `localhost`
and `.local`/`.internal` names are rejected.

**Default list.** On first boot the file is seeded from `agent.yaml` with what
the installed tools need: Anthropic and Claude, OpenAI and ChatGPT, GitHub
(including raw content and ghcr.io), Homebrew, npm and nodejs.org, PyPI, Maven
Central, Clojars, mise, Docker Hub, and ubuntu.com for apt and time sync. The
file is never overwritten afterwards, so edits survive reboots; a recreated VM
starts from the seed again.

**Editing.**

```bash
lmadmin sudo nano /etc/agent-egress/allowlist.txt   # or: lmadmin sudo tee ... < my-list.txt
lmadmin sudo agent-egress-reload
```

The reload takes about a second. It restarts dnsmasq and empties the set of
admitted addresses, so a removed name stops working as soon as its open
connections close; kept names are re-admitted on their next lookup.

**Allow everything for a while.** Add a line containing only `*` and reload:
every name resolves and every address is admitted, but each lookup is still
logged. Use it to discover what a new tool needs, then build the list from the
journal, delete the `*` line, and reload:

```bash
limactl shell sandbox journalctl -t dnsmasq -o cat | awk '$1 ~ /^query/ { print $2 }' | sort -u
```

See what was blocked (the agent can run these too):

```bash
limactl shell sandbox journalctl -t dnsmasq -o cat -f | grep NXDOMAIN   # unlisted names
limactl shell sandbox journalctl -k -o cat -f | grep agent-egress       # rejected connections
```

## Verify

Inside the VM as `agent` (`limactl shell sandbox`), with `github.com` on the list:

```bash
curl -fsS -o /dev/null https://api.github.com/ && echo allowed-ok
curl -sS https://example.com/; echo "exit $? (want 6: could not resolve)"
curl -sS --max-time 5 https://1.1.1.1/; echo "exit $? (want 7: connection refused)"
curl -sS --max-time 5 "https://$(getent ahostsv4 github.com | awk 'NR==1{print $1}'):8443/"; echo "exit $? (want 7)"
getent hosts example.com || echo nxdomain-ok
docker run --rm curlimages/curl:latest -fsS -o /dev/null https://api.github.com/ && echo container-ok
docker run --rm curlimages/curl:latest -sS https://example.com/ || echo container-blocked-ok
```

## Limits

- An admitted address is reachable on 80/443 with any TLS name: a script that
  targets a CDN address shared with an allowlisted host can reach other tenants
  of that address. Closing this needs a proxy in the TLS path (see `../spike`).
- Admitted addresses stay admitted until the next reload or reboot.
- IPv4 only outside the VM; IPv6 loopback still works.
- Allowed hosts remain exfiltration channels. Scope your tokens.
