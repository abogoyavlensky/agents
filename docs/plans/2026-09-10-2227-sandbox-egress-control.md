# Sandbox Egress Control Implementation Plan

> **For agentic workers:** Use executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restrict outbound network access of AI coding agents inside the Lima sandbox VM to an allowlist of hostnames, enforced by nftables and a local mitmproxy, with the `agent` user stripped of sudo.

**Tech Stack:** Lima (vz, Ubuntu 24.04 guest), nftables, mitmproxy 12.2.3 standalone binary, systemd, Python 3 (mitmproxy addon, stdlib `unittest`), bash provisioning in `sandbox/agent.yaml`.

---

## Design

### Threat model and scope

The agents (Claude Code, Codex, opencode, pi) run with permission prompts disabled inside a Lima VM. The risk this plan addresses is a prompt-injected agent sending tokens or source to an arbitrary host, or fetching and running code from an arbitrary URL. It does not address damage through allowed hosts (force pushes, API calls with real credentials); token scoping and agent hooks cover that separately.

Out of scope: the Linux dev servers in `server/` (a later plan can port this design), per-tool sandboxes, TLS interception, URL-level rules.

### Boundary

The boundary is a Unix user, consistent with `sandbox/fresh_server_setup.md`:

- `agent` loses sudo. It keeps exactly one sudo command, `cat /mnt/lima-cidata/param.env`, which Lima's host agent runs over ssh on every start when checking readiness. Everything else is denied by a root-owned sudoers file sorted last (`/etc/sudoers.d/zz-agent-egress`). Last match wins in sudoers, so the denial holds even during the boot window where cloud-init re-creates its `90-cloud-init-users` grant (Lima regenerates the cloud-init instance id on every start, so cloud-init's user module re-runs each boot).
- `admin` is a new human account with passwordless sudo. Its ssh key is extracted from `/mnt/lima-cidata/user-data` (root-only), never from the agent's writable `authorized_keys`. Reached from the Mac with `ssh -F ~/.lima/sandbox/ssh.config -o ControlPath=none -l admin lima-sandbox`. `ControlPath=none` matters: Lima multiplexes ssh over a control socket and reusing it would land in the agent's authenticated session.
- `agent-proxy` is a static system user that runs mitmproxy. Static, not `DynamicUser`, because nftables needs a fixed uid to forbid the proxy from reaching private address ranges.

Lima facts this relies on (verified against Lima source and templates):

- Provision scripts run on every boot, in order: cloud-init user setup, Lima boot scripts, `mode: data` files, `mode: system` scripts, `mode: user` scripts (run by root via `sudo -iu agent`, so agent needs no sudo for them).
- `mode: data` files with `file:` are read from paths relative to the template at `limactl create` time and embedded in the instance's `lima.yaml`.
- The yaml `env:` map is written to `/etc/environment` by Lima. `propagateProxyEnv` defaults to true and must be set false so the Mac's environment never overrides ours.
- The agent's home is `/home/agent.guest` (Lima default `{{.Home}}.guest`). Scripts use `$HOME` or `getent`, never a hardcoded path.
- `template:docker` installs rootless Docker. No docker group is involved. Rootless Docker allocates a subordinate uid range to `agent` in `/etc/subuid`.

### Network enforcement (nftables)

Table `inet agent-egress`, loaded early by `nftables.service` from `/etc/nftables.conf` and re-applied by provisioning with an atomic `nft -f` (the file starts with `table inet agent-egress {}` then `flush table inet agent-egress`, never `flush ruleset`).

First rule in the chain: `ct state established,related accept`. The output hook also sees locally generated reply packets, so without this the proxy's replies to its clients on 127.0.0.1:8080 would hit the proxy's private-range reject below. Every rule after it therefore only judges new connections.

Agent uids, a set containing the `agent` uid and its full `/etc/subuid` range (a user namespace can create sockets owned by subordinate uids, which would otherwise bypass a single-uid match):

- output on `lo`: accept. Dev servers, nREPL, opencode's local server and the proxy all live on loopback. With systemd-resolved disabled (below) nothing on loopback can relay for the agent except the proxy.
- everything else: reject, with rate-limited logging (`limit rate 10/second` on the log rule, then an unconditional reject). TCP is rejected `with tcp reset` so clients see connection refused instantly; other protocols get the default ICMP unreachable.

Proxy uid:

- UDP and TCP 53 to the Lima resolver: accept.
- destinations in `10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, 127.0.0.0/8, 100.64.0.0/10` and IPv6 `::1, fc00::/7, fe80::/10`: reject. This stops an allowed hostname that resolves to the Mac (192.168.5.2) or the VM from being reached through the proxy.

Everything else keeps the default accept policy.

### DNS

systemd-resolved is disabled and `/etc/resolv.conf` becomes a plain file pointing at the Lima resolver (`$LIMA_CIDATA_SLIRP_DNS`, normally 192.168.5.3). Reason: resolved exposes a world-accessible IPC socket and forwards queries under its own uid, so an agent could exfiltrate data as DNS labels even with its own port 53 traffic blocked. Without resolved, the agent's lookups go straight to UDP 53 and are rejected. Root, the proxy and other system users still resolve normally. Proxy clients do not need local DNS because they send the hostname in CONNECT.

Fallback if Lima's boot fights this (resolv.conf rewritten, DNS broken for root): leave resolved enabled, drop the resolv.conf steps, and record the residual DNS channel in the backlog.

### Proxy

mitmproxy 12.2.3 standalone tarball for the VM's architecture (`uname -m` gives `aarch64` or `x86_64`, matching the release names `mitmproxy-12.2.3-linux-<arch>.tar.gz` from `downloads.mitmproxy.org`). Installed under `/opt/mitmproxy/12.2.3/` with a `/opt/mitmproxy/current` symlink. `mitmdump` runs as a systemd system service (`agent-proxy.service`) as user `agent-proxy`, listening on 127.0.0.1:8080, explicit proxy mode, `connection_strategy=lazy` so no upstream connection is opened before policy runs, `confdir` under `/var/lib/agent-proxy`. No mitmweb: with TLS passthrough it shows nothing useful and is an admin interface the agent could reach on loopback.

The addon `/etc/agent-proxy/allowlist_addon.py` enforces policy without TLS interception:

- `http_connect`: set a 403 response first, then evaluate; clear the response only when allowed. Allowed means: host is a hostname (IP literals always denied), not `localhost`, `*.localhost`, `*.local`, `*.internal`, port is 443, and the host matches the allowlist.
- `request` (plain HTTP): same host rules, port 80 only. Also runs for requests inside any tunnel mitmproxy ends up intercepting; those are denied.
- `tls_clienthello`: compare the ClientHello SNI with the CONNECT host. Equal: set `data.ignore_connection = True`, bytes pass through untouched. Missing or different: leave interception on and log `sni-mismatch`. Because no client in the VM trusts mitmproxy's CA, interception makes the handshake fail, which denies the connection. This closes the shared-CDN gap where a client CONNECTs to an allowed Cloudflare-fronted host and puts an attacker's name in the handshake.
- Allowlist file `/etc/agent-proxy/allowlist.txt`: one entry per line, `#` comments. An entry matches itself and all subdomains. A line containing only `*` means allow every hostname (discovery mode). Missing, empty, unreadable or unparsable file means deny all. Re-read when its mtime changes.
- Decision log `/var/log/agent-proxy/decisions.log`: one line per decision, `<iso-timestamp> <allow|deny> <connect|http|sni> <host>:<port> <reason>`.
- Fail closed: hooks set the deny outcome before any fallible work, so an exception inside the addon leaves the request denied.
- Single file with a guarded `mitmproxy` import so the pure policy functions (`parse_rules`, `decide`) are unit-testable on a machine without mitmproxy.

### Tool configuration

Proxy variables live in `/etc/profile.d/agent-proxy.sh` (root-owned, installed as a `mode: data` file), guarded so they are exported only for non-root users:

```
HTTP_PROXY / http_proxy   = http://127.0.0.1:8080
HTTPS_PROXY / https_proxy = http://127.0.0.1:8080
NO_PROXY / no_proxy       = localhost,127.0.0.1,::1
NODE_USE_ENV_PROXY        = 1
JAVA_TOOL_OPTIONS         = -Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=8080 -Dhttps.proxyHost=127.0.0.1 -Dhttps.proxyPort=8080 -Dhttp.nonProxyHosts=localhost|127.0.0.1
```

Why not Lima's `env:` field: Lima writes `env:` to `/etc/environment` and exports it before running provision scripts, so root's bootstrap steps (apt-get, the docker template's `get.docker.com` download, the mitmproxy download itself) would try to use a proxy that does not exist yet on first boot. profile.d is read only by login shells, which root's provisioning shell is not. All agent entry points are login shells: `limactl shell` runs `$SHELL -l` (also with `-c` for commands, which is why the existing `.bash_profile` PATH setup works), `mode: user` provisioning runs via `sudo -iu agent`, and interactive ssh logs in. A non-login session without the variables fails fast with connection refused rather than silently bypassing anything. `propagateProxyEnv: false` is still set so the Mac's proxy variables never reach the VM.

Lowercase variants exist because curl deliberately ignores uppercase `HTTP_PROXY`. `JAVA_TOOL_OPTIONS` makes the JVM print one "Picked up JAVA_TOOL_OPTIONS" line to stderr per launch; the README documents how to unset it if that annoys.

Maven's resolver (used by the Clojure CLI) ignores JVM proxy properties, so `mode: user` provisioning writes `~/.m2/settings.xml` with a proxy block if the file does not exist.

Homebrew's installer needs sudo to create `/home/linuxbrew/.linuxbrew` on first boot. The system script pre-creates it owned by `agent` before user provisioning runs.

### Rootless Docker

The daemon runs in rootlesskit's network namespace, where 127.0.0.1 is not the VM's loopback. Rootlesskit exposes the VM's loopback as 10.0.2.2 (slirp4netns) but disables it by default. A user-level drop-in `~/.config/systemd/user/docker.service.d/agent-egress.conf` sets `DOCKERD_ROOTLESS_ROOTLESSKIT_DISABLE_HOST_LOOPBACK=false` and the proxy variables pointing at `http://10.0.2.2:8080`, so image pulls go through the allowlist. `~/.docker/config.json` gets a `proxies.default` block with the same values so containers and builds inherit them. Container traffic that ignores the proxy exits through slirp4netns as an agent uid and is rejected; containers have no external DNS, by design. Testcontainers, postgres and localstack work: they use the agent's docker socket, mapped ports on loopback, and pulls through the daemon.

Verify in the VM whether the network driver is slirp4netns or pasta (`docker info` or the rootlesskit process arguments); if pasta, confirm the host-loopback address before trusting 10.0.2.2.

### Allowlist workflow

The repo file `sandbox/egress/allowlist.txt` is the seed, installed with `overwrite: false` so admin edits in the VM persist across reboots. The addon picks up edits on mtime change. To push the repo version: `lmadmin sudo tee /etc/agent-proxy/allowlist.txt < sandbox/egress/allowlist.txt`. Never read the allowlist from the writable Projects mount: the agents repo lives under it, so the agent can edit that copy.

Phase one ships `*` (log everything). After a week of use, `sort -u` the hosts from the decision log into the allowlist and remove `*`.

### Verification

`agent-egress-verify`, installed to `/usr/local/bin`, run as `agent` inside the VM:

1. `sudo -n true` fails.
2. `sudo -n cat /mnt/lima-cidata/param.env` succeeds.
3. `curl -sS --max-time 5 https://example.com` succeeds (allowlist contains `*` or `example.com`).
4. `curl -sS --noproxy '*' --max-time 5 https://1.1.1.1` exits 7 (connection refused). A numeric address, because the agent has no DNS and a hostname would fail with exit 6 without exercising the firewall. Exit 28 (timeout) is a failure of this check.
5. `curl -sS --max-time 5 -x http://127.0.0.1:8080 https://127.0.0.1/` gets HTTP 403 from the proxy.
6. `curl -sS --max-time 5 -x http://127.0.0.1:8080 http://example.com:8443/` gets 403 (port rule).
7. `getent hosts example.com` fails.
8. Direct TCP to a numeric address on port 22 is refused within 3 seconds, for example `timeout 3 bash -c 'exec 3<>/dev/tcp/140.82.121.4/22'` exits non-zero and quickly.
9. The decision log has an `allow connect example.com:443` line.
10. SNI mismatch is denied: `curl -sS --max-time 5 -x http://127.0.0.1:8080 --connect-to example.com:443:example.org:443 https://example.com` fails the TLS handshake (non-zero exit) and the decision log gains a `deny sni` line for `example.org`.

Unit tests for the addon's pure functions run on any machine with Python 3. Everything else can only be exercised in a real Lima VM on the Mac; the final task is manual.

### Known limitations (record in README)

- Only HTTPS on 443 and HTTP on 80 through the proxy. ssh-based git, ping, raw DNS and other protocols are unavailable to the agent. Git uses HTTPS with the forwarded `GH_TOKEN`.
- Allowed hosts remain exfiltration channels (a gist on github.com). Token scoping and hooks are the controls for that.
- Containers have no external DNS and must use the proxy.
- Any tool that ignores proxy environment variables fails fast until its own proxy knob is set.

## File Structure

- Modify: `sandbox/agent.yaml`. Adds `propagateProxyEnv: false`, `mode: data` entries for the files below, one `mode: system` script (users, sudo, DNS, Homebrew prefix, nftables, mitmproxy install, service enablement) and one `mode: user` script (Maven settings, Docker drop-in and config.json). The existing system and user scripts are left untouched; the Homebrew prefix fix lives in the new system script.
- Create: `sandbox/egress/sudoers-agent` (installed as `/etc/sudoers.d/zz-agent-egress`, root, 0440).
- Create: `sandbox/egress/sudoers-admin` (installed as `/etc/sudoers.d/zz-admin`, root, 0440).
- Create: `sandbox/egress/profile-agent-proxy.sh` (installed as `/etc/profile.d/agent-proxy.sh`, root, 0644).
- Create: `sandbox/egress/nftables.conf.tmpl` (installed as `/etc/agent-proxy/nftables.conf.tmpl`; the system script substitutes `@AGENT_UID@`, `@SUBUID_START@`, `@SUBUID_END@`, `@PROXY_UID@`, `@RESOLVER@` into `/etc/nftables.conf`).
- Create: `sandbox/egress/allowlist_addon.py` (installed as `/etc/agent-proxy/allowlist_addon.py`).
- Create: `sandbox/egress/test_allowlist_addon.py` (repo only, not installed).
- Create: `sandbox/egress/allowlist.txt` (installed as `/etc/agent-proxy/allowlist.txt`, `overwrite: false`).
- Create: `sandbox/egress/agent-proxy.service` (installed as `/etc/systemd/system/agent-proxy.service`).
- Create: `sandbox/egress/verify.sh` (installed as `/usr/local/bin/agent-egress-verify`, 0755).
- Modify: `sandbox/README.md`. Admin alias, allowlist workflow, verification, limitations, JVM note.
- Modify: `.gitignore`. Add `.tmp/` (Codex consultation scratch files).

## Tasks

### Task 1: Housekeeping and sudoers files

**Files:**
- Modify: `.gitignore`
- Create: `sandbox/egress/sudoers-agent`
- Create: `sandbox/egress/sudoers-admin`

- [x] **Step 1: Ignore scratch files**
  Append `.tmp/` to `.gitignore` under a short comment.

- [x] **Step 2: Write the agent sudoers file**
  Two lines. First denies everything: `agent ALL=(ALL:ALL) !ALL`. Second allows the Lima probe command without a password for both path spellings: `agent ALL=(root) NOPASSWD: /usr/bin/cat /mnt/lima-cidata/param.env, /bin/cat /mnt/lima-cidata/param.env`. Order matters: last match wins, so the allow comes after the deny. Add a comment explaining why the file name sorts after `90-cloud-init-users`.

- [x] **Step 3: Write the admin sudoers file**
  One line: `admin ALL=(ALL:ALL) NOPASSWD:ALL`.

- [x] **Step 4: Validate syntax**
  Run: `visudo -cf sandbox/egress/sudoers-agent && visudo -cf sandbox/egress/sudoers-admin`
  Expected: both report `parsed OK`. If `visudo` is unavailable on this machine, note it and rely on the in-VM check in Task 8.

- [x] **Step 5: Commit**
  `git commit -am "Add sudoers files for sandbox egress control"` (add the new files first).

### Task 2: nftables ruleset template

**Files:**
- Create: `sandbox/egress/nftables.conf.tmpl`

- [x] **Step 1: Write the template**
  Shebang `#!/usr/sbin/nft -f`. Then, in order: `table inet agent-egress {}`, `flush table inet agent-egress`, and the table definition with:
  - `set agent_uids { type uid; flags interval; elements = { @AGENT_UID@, @SUBUID_START@-@SUBUID_END@ } }`
  - chain `output`, `type filter hook output priority filter; policy accept;`
  - first rule: `ct state established,related accept` (reply packets, including the proxy's replies to loopback clients, must never be judged by the rules below).
  - agent rules: `meta skuid @agent_uids oifname "lo" accept`; `meta skuid @agent_uids limit rate 10/second log prefix "agent-egress "`; `meta skuid @agent_uids meta l4proto tcp counter reject with tcp reset`; `meta skuid @agent_uids counter reject`.
  - proxy rules: accept `meta skuid @PROXY_UID@ ip daddr @RESOLVER@ udp dport 53` and the same for `tcp dport 53`; reject `meta skuid @PROXY_UID@ ip daddr { 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, 127.0.0.0/8, 100.64.0.0/10 } counter reject`; reject `meta skuid @PROXY_UID@ ip6 daddr { ::1, fc00::/7, fe80::/10 } counter reject`.
  Comment each block with one line saying what it protects.

- [x] **Step 2: Syntax-check with placeholder values**
  `nft -c -f` needs CAP_NET_ADMIN and will fail on this machine with `Operation not permitted`. Instead run a substitution dry run to confirm every placeholder is replaced:
  Run: `sed -e 's/@AGENT_UID@/1000/' -e 's/@SUBUID_START@/100000/' -e 's/@SUBUID_END@/165535/' -e 's/@PROXY_UID@/998/' -e 's/@RESOLVER@/192.168.5.3/' sandbox/egress/nftables.conf.tmpl | grep -c '@'`
  Expected: `0`. The real `nft -c` check happens in Task 8.

  > Deviation: the check uses `grep -c '@[A-Z_]\+@'` rather than `grep -c '@'`, because nftables set references (`@agent_uids`) legitimately contain `@` and would never reach zero. `nft -c` was also attempted here and, as the plan predicted, fails without CAP_NET_ADMIN (`unshare -rn` is unavailable in this container too), so real ruleset validation still happens in Task 8.

- [x] **Step 3: Commit**
  `git add sandbox/egress/nftables.conf.tmpl && git commit -m "Add nftables egress ruleset template"`

### Task 3: mitmproxy allowlist addon (TDD)

**Files:**
- Create: `sandbox/egress/allowlist_addon.py`
- Test: `sandbox/egress/test_allowlist_addon.py`

The pure functions the tests and the hooks share:

```python
def parse_rules(text: str) -> "Rules"        # Rules has .allow_all: bool and .domains: frozenset[str]
def decide(host: str, port: int, kind: str, rules: "Rules | None") -> tuple[bool, str]
    # kind is "connect" or "http"; returns (allowed, reason)
```

- [x] **Step 1: Write failing tests**
  `unittest` cases for `parse_rules`: comments and blanks ignored, entries lowercased and stripped of a leading dot, `*` sets `allow_all`, empty text gives no domains and `allow_all` false. Cases for `decide`: exact match allowed; subdomain allowed; sibling domain (`notgithub.com` vs `github.com`) denied; IP literal denied even with `allow_all`; `localhost`, `foo.local`, `foo.internal`, `host.lima.internal` denied even with `allow_all`; connect on port 8443 denied; http on port 8080 denied; `rules is None` denies everything with reason `no-allowlist`; each denial returns a distinct reason string.

- [x] **Step 2: Run tests to verify they fail**
  Run: `cd sandbox/egress && python3 -m unittest -v test_allowlist_addon`
  Expected: FAIL with `ModuleNotFoundError` or `ImportError`.

- [x] **Step 3: Implement the addon**
  Top of file: `try: from mitmproxy import ctx, http, tls` guarded by `except ImportError`, setting the names to `None`, so the module imports without mitmproxy. Implement `parse_rules` and `decide` as specified in the Design section. Then the addon class:
  - `load(loader)`: add options `allowlist` (path) and `decision_log` (path).
  - `configure(updates)`: (re)load rules; on any error set rules to `None` and log a warning.
  - `_rules()`: reload if the allowlist mtime changed; return current rules.
  - `_log(decision, kind, host, port, reason)`: append one line to the decision log, flush.
  - `http_connect(flow)`: `flow.response = http.Response.make(403, b"blocked by agent-proxy allowlist\n")` first, then `decide(...)`; on allow set `flow.response = None`. Log.
  - `request(flow)`: same for plain HTTP using `flow.request.pretty_host` and `flow.request.port`, kind `http`.
  - `tls_clienthello(data)`: `connect_host = data.context.server.address[0]`, `sni = data.client_hello.sni`; if equal (case-insensitive) set `data.ignore_connection = True` and log `allow sni`; otherwise log `deny sni <reason>` and leave `ignore_connection` false.
  - `addons = [AllowlistAddon()]` at module bottom, created only when mitmproxy imported.

- [x] **Step 4: Run tests to verify they pass**
  Run: `cd sandbox/egress && python3 -m unittest -v test_allowlist_addon`
  Expected: all PASS.

- [x] **Step 5: Byte-compile check**
  Run: `python3 -m py_compile sandbox/egress/allowlist_addon.py && echo ok`
  Expected: `ok`.

- [x] **Step 6: Commit**
  `git add sandbox/egress/allowlist_addon.py sandbox/egress/test_allowlist_addon.py && git commit -m "Add mitmproxy allowlist addon with tests"`

  > Deviation: both hooks read `flow.request.host`, not `flow.request.pretty_host` as the plan specified. Codex found that `pretty_host` prefers the client-supplied `Host` header, so `CONNECT attacker.example:443` with `Host: github.com` would have been judged against the header while mitmproxy routed to the attacker. Fixed in `84fd2f2`.
  > Deviation: `parse_rules` also accepts a leading `*.` on an entry (`*.github.com` becomes `github.com`), with a test, so the common spelling is not silently a no-op.

### Task 4: Allowlist seed, profile script, systemd unit, verify script

**Files:**
- Create: `sandbox/egress/allowlist.txt`
- Create: `sandbox/egress/profile-agent-proxy.sh`
- Create: `sandbox/egress/agent-proxy.service`
- Create: `sandbox/egress/verify.sh`

- [x] **Step 0: Write the profile script**
  POSIX sh (profile.d scripts are sourced by dash on Ubuntu). `[ "$(id -u)" -eq 0 ] && return` guard first, then `export` the variables from the Design section's Tool configuration table, both cases. A comment on the `JAVA_TOOL_OPTIONS` line saying it may be commented out if the stderr notice annoys. Check with `sh -n sandbox/egress/profile-agent-proxy.sh`.

- [x] **Step 1: Write the allowlist seed**
  Header comment explaining the format (one host per line, matches subdomains, `*` means allow all, missing or empty file means deny all, edits picked up automatically). Active content: a single `*` line. Below it, a commented block titled "Starting point after discovery" listing the known hosts so the user can uncomment them later: `api.anthropic.com`, `claude.ai`, `platform.claude.com`, `downloads.claude.ai`, `code.claude.com`, `statsig.anthropic.com`, `sentry.io`, `chatgpt.com`, `api.openai.com`, `auth.openai.com`, `github.com`, `api.github.com`, `objects.githubusercontent.com`, `raw.githubusercontent.com`, `codeload.github.com`, `ghcr.io`, `registry.npmjs.org`, `pypi.org`, `files.pythonhosted.org`, `repo1.maven.org`, `repo.clojars.org`, `formulae.brew.sh`, `mise.run`, `mise.jdx.dev`, `registry-1.docker.io`, `auth.docker.io`, `production.cloudflare.docker.com`. Mark the list as unverified until the decision log confirms it.

- [x] **Step 2: Write the systemd unit**
  `[Unit]`: description, `After=network-online.target nftables.service`, `Wants=network-online.target`. `[Service]`: `User=agent-proxy`, `Group=agent-proxy`, `StateDirectory=agent-proxy`, `LogsDirectory=agent-proxy`, `Environment=HOME=/var/lib/agent-proxy`, `ExecStart=/opt/mitmproxy/current/mitmdump --listen-host 127.0.0.1 --listen-port 8080 --set confdir=/var/lib/agent-proxy --set connection_strategy=lazy --set termlog_verbosity=warn -s /etc/agent-proxy/allowlist_addon.py --set allowlist=/etc/agent-proxy/allowlist.txt --set decision_log=/var/log/agent-proxy/decisions.log`, `Restart=always`, `RestartSec=2`, `NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, `PrivateTmp=yes`. `[Install]`: `WantedBy=multi-user.target`. Do not pass any proxy environment to this service.

- [x] **Step 3: Write the verify script**
  Bash, `set -u`, a `check` helper that prints `PASS`/`FAIL` with the check name and counts failures, exits non-zero if any failed. Implement the ten checks from the Design section's Verification list. Direct-connection checks use numeric addresses only. Check 4 must distinguish connection refused (curl exit 7) from timeout (exit 28) and fail on timeout. Read the allowlist and print `SKIP` for checks 3, 9 and 10 if neither `*` nor `example.com` (and `example.org` for check 10) is present.

- [x] **Step 4: Static checks**
  Run: `bash -n sandbox/egress/verify.sh && (command -v shellcheck >/dev/null && shellcheck sandbox/egress/verify.sh || echo "shellcheck not installed") && systemd-analyze verify sandbox/egress/agent-proxy.service 2>&1 | grep -v 'agent-proxy.service:.*Unit.*not found' ; echo done`
  Expected: no syntax errors; `systemd-analyze` may warn about the missing ExecStart binary on this machine, which is fine.

- [x] **Step 5: Commit**
  `git add sandbox/egress && git commit -m "Add allowlist seed, proxy service unit and verify script"`

  > Deviation: check 5 passes `--noproxy ''` as well as `-x`. Codex found that curl honours `NO_PROXY` (which lists `127.0.0.1`) even when a proxy is given explicitly, so the check would have connected directly and failed on a healthy sandbox. Fixed in `79d1106`.
  > Deviation: the `check` helper captures its function's status with `"$fn" || rc=$?` rather than reading `$?` after an `if`, where it is the compound statement's own status. Without this the SKIP path never fired and skipped checks were reported as failures.
  > Deviation: `.gitignore` also ignores `__pycache__/`, since running the addon tests creates one next to the sources.

### Task 5: agent.yaml provisioning

**Files:**
- Modify: `sandbox/agent.yaml`

- [x] **Step 1: Add Lima settings**
  Below `user:` add `propagateProxyEnv: false`. Do not add an `env:` block: the proxy variables come from the profile.d file (see Design, Tool configuration) so root's bootstrap steps never inherit them.

- [x] **Step 2: Add `mode: data` entries**
  One entry per installed file, before the scripts in the `provision` list, each with `file:` pointing at `egress/<name>` (relative to the template), `path`, `owner`, `permissions`:
  - `egress/sudoers-agent` to `/etc/sudoers.d/zz-agent-egress`, `root:root`, `440`
  - `egress/sudoers-admin` to `/etc/sudoers.d/zz-admin`, `root:root`, `440`
  - `egress/profile-agent-proxy.sh` to `/etc/profile.d/agent-proxy.sh`, `root:root`, `644`
  - `egress/nftables.conf.tmpl` to `/etc/agent-proxy/nftables.conf.tmpl`, `644`
  - `egress/allowlist_addon.py` to `/etc/agent-proxy/allowlist_addon.py`, `644`
  - `egress/allowlist.txt` to `/etc/agent-proxy/allowlist.txt`, `644`, `overwrite: false`
  - `egress/agent-proxy.service` to `/etc/systemd/system/agent-proxy.service`, `644`
  - `egress/verify.sh` to `/usr/local/bin/agent-egress-verify`, `755`

- [x] **Step 3: Add the egress `mode: system` script**
  Place it after the existing system script. `set -eux`. Sections, each idempotent:
  1. Users: `id admin || useradd -m -s /bin/bash admin`; `getent passwd agent-proxy || useradd --system --no-create-home --shell /usr/sbin/nologin agent-proxy`.
  2. Admin key: create `/home/admin/.ssh` (0700, admin); extract keys with `grep -oE '"(ssh-|ecdsa-)[^"]+"' /mnt/lima-cidata/user-data | tr -d '"'` into `authorized_keys` (0600, admin). Fail loudly if zero keys were extracted.
  3. Agent sudo: `rm -f /etc/sudoers.d/90-cloud-init-users`; `gpasswd -d agent sudo 2>/dev/null || true`; `visudo -cf /etc/sudoers.d/zz-agent-egress`.
  4. Homebrew prefix: `install -d -o agent -g agent /home/linuxbrew/.linuxbrew`.
  5. DNS: if `systemd-resolved` is enabled, `systemctl disable --now systemd-resolved`; if `/etc/resolv.conf` is a symlink or lacks the resolver line, replace it with `nameserver ${LIMA_CIDATA_SLIRP_DNS:-192.168.5.3}`.
  6. nftables: `apt-get install -y nftables` if `nft` missing; compute `AGENT_UID=$(id -u agent)`, `PROXY_UID=$(id -u agent-proxy)`, subuid range from `awk -F: '$1=="agent"{print $2, $2+$3-1}' /etc/subuid` (fail loudly if empty), `RESOLVER` as in step 5; `sed` the placeholders from the template into `/etc/nftables.conf`; `nft -c -f /etc/nftables.conf`; `nft -f /etc/nftables.conf`; `systemctl enable nftables`.
  7. mitmproxy: `VER=12.2.3`; `ARCH=$(uname -m)`; if `/opt/mitmproxy/$VER/mitmdump` missing, download `https://downloads.mitmproxy.org/$VER/mitmproxy-$VER-linux-$ARCH.tar.gz` to a temp dir, extract into `/opt/mitmproxy/$VER`, `ln -sfn /opt/mitmproxy/$VER /opt/mitmproxy/current`.
  8. Service: `systemctl daemon-reload`; `systemctl enable --now agent-proxy`; also `unset` any `*_proxy`/`*_PROXY` variables at the very top of this script as belt and braces, so the mitmproxy download never depends on the proxy it installs; `systemctl restart agent-proxy` only if the unit or addon changed (compare `systemctl show -p NeedDaemonReload` or simply always restart; always restarting is acceptable and simpler); wait until `ss -ltn` shows 127.0.0.1:8080 or fail after 15 seconds.

- [x] **Step 4: Add the egress `mode: user` script**
  Place it after the existing user script. `set -eux`.
  1. Maven: if `$HOME/.m2/settings.xml` is missing, write it with one `<proxy>` for `http` and one for `https`, host 127.0.0.1, port 8080, `nonProxyHosts` `localhost|127.0.0.1`.
  2. Docker daemon drop-in: write `$HOME/.config/systemd/user/docker.service.d/agent-egress.conf` with `[Service]` and `Environment=` lines for `DOCKERD_ROOTLESS_ROOTLESSKIT_DISABLE_HOST_LOOPBACK=false`, `HTTP_PROXY`, `HTTPS_PROXY`, `http_proxy`, `https_proxy` set to `http://10.0.2.2:8080`, and `NO_PROXY`/`no_proxy` set to `localhost,127.0.0.1,::1,10.0.2.2`. Only when the content changed: `systemctl --user daemon-reload` and, if `docker.service` exists, `systemctl --user restart docker`.
  3. Docker client: merge a `proxies.default` object (`httpProxy`, `httpsProxy` = `http://10.0.2.2:8080`, `noProxy` = `localhost,127.0.0.1,::1,10.0.2.2`) into `$HOME/.docker/config.json` with `jq`, creating the file if missing.

- [x] **Step 5: Validate YAML and embedded shell**
  Run: `python3 -c "import yaml,sys; d=yaml.safe_load(open('sandbox/agent.yaml')); print(len(d['provision']), 'provision entries'); [print(p['mode'], p.get('path','')) for p in d['provision']]"`
  Expected: the list shows the data entries, two system scripts and two user scripts.
  Then extract each script and syntax-check it:
  Run: `python3 -c "import yaml; d=yaml.safe_load(open('sandbox/agent.yaml')); [open(f'/tmp/prov{i}.sh','w').write(p['script']) for i,p in enumerate(d['provision']) if 'script' in p]" && for f in /tmp/prov*.sh; do bash -n "$f" && echo "$f ok"; done`
  Expected: every script prints `ok`.

- [x] **Step 6: Commit**
  `git commit -am "Provision egress control in the Lima sandbox"`

  > Deviation: the nftables output chain runs at `priority -140`, not `priority filter` (0). Lima's own `LIMADNS` chain DNATs `192.168.5.3:53` to `<slirp gateway>:<random port>` in the nat output hook at priority -100 (`boot.Linux/09-host-dns-setup.sh`), so at priority 0 the resolver-accept rule would never match and the private-range reject would have killed the proxy's DNS. -140 sits after conntrack (-200), so `ct state` still works, and before nat, so every `ip daddr` is the destination the socket asked for.
  > Deviation: `/home/linuxbrew` stays root-owned and the script refuses a symlinked `.linuxbrew`. Codex found that with the agent owning the parent, it could swap `.linuxbrew` for a symlink between boots, and `install -d` (verified locally) follows symlinks - handing the agent ownership of the target on the next root re-run. Fixed in `38b2d3f`.
  > Deviation: the admin key extraction pattern is `"(sk-)?(ssh-|ecdsa-)[^"]+"`, covering FIDO `sk-` key types as well.

  Lima behaviour verified against the upstream source rather than assumed: `mode: data` `file:` entries are embedded as `content` at create time (`pkg/limatmpl/embed.go`), data files are installed with parent directories created and `overwrite: false` honoured before any provision script (`cidata.TEMPLATE.d/boot.sh`), `mode: user` scripts run as `sudo -iu agent` (a login shell, so profile.d applies), and the host agent's readiness probe really does run `sudo cat /mnt/lima-cidata/param.env` (`pkg/hostagent/requirements.go`).

### Task 6: README

**Files:**
- Modify: `sandbox/README.md`

- [x] **Step 1: Add an "Egress control" section**
  Cover, in this order and briefly: what is enforced (agent uid to loopback only, proxy allowlist, no sudo); the `lmadmin` alias `ssh -F ~/.lima/sandbox/ssh.config -o ControlPath=none -l admin lima-sandbox`; the allowlist workflow (seed with `*`, read `/var/log/agent-proxy/decisions.log`, edit in the VM or push from the repo with the `tee` one-liner, mtime reload); running `agent-egress-verify` after every VM recreation; the JVM stderr note and how to unset `JAVA_TOOL_OPTIONS`; the Docker notes (pulls and containers go through 10.0.2.2:8080, no external DNS inside containers); the known limitations list from the Design section.

- [x] **Step 2: Note the recreate requirement**
  Because `mode: data` files are embedded at create time, changes to `sandbox/egress/*` reach an existing instance only via `limactl delete` and `limactl create`, or by editing the embedded copy with `limactl edit`. Say so in one sentence.

- [x] **Step 3: Commit**
  `git commit -am "Document sandbox egress control"`

  > Deviation: the discovery one-liner in the README strips the port and filters to allowed connect/http lines. Codex found that the decision log's fourth field is `host:port`, so the naive `awk '{print $4}'` would have produced entries the addon can never match, silently blocking everything once `*` was removed. Fixed in `443691f`.

### Task 7: Codex plan and code review checkpoint

- [ ] **Step 1: Run /review-with-codex on the branch against master**
  Address real findings, especially anything about fail-open paths in the addon or the provisioning order.

- [ ] **Step 2: Commit fixes**
  `git commit -am "Address review findings for egress control"` if there were any.

### Task 8: Manual verification on the Mac (human)

This task cannot run from this machine. The executor stops here and hands the checklist to the user.

- [ ] **Step 1: Recreate the instance**
  `limactl stop sandbox; limactl delete sandbox; limactl create --name sandbox sandbox/agent.yaml; limactl start sandbox` then re-add the Projects mount per the README. Watch `limactl start` for provisioning errors.

- [ ] **Step 2: Admin access**
  `ssh -F ~/.lima/sandbox/ssh.config -o ControlPath=none -l admin lima-sandbox sudo -n true` succeeds.

- [ ] **Step 3: Ruleset and services**
  As admin: `sudo nft list table inet agent-egress` shows the set with two uid intervals; `systemctl is-active agent-proxy nftables` prints `active` twice; `resolvectl` is gone or inactive and `cat /etc/resolv.conf` shows the Lima resolver; `getent hosts example.com` works as admin.

- [ ] **Step 4: Verify script**
  `limactl shell sandbox agent-egress-verify` prints all PASS. Also `limactl shell sandbox env | grep -i proxy` shows the variables (confirms the login-shell path), and as admin `sudo cat /var/log/agent-proxy/decisions.log | tail` shows the allow and deny lines from the script.

- [ ] **Step 5: Agents and tools**
  Run `lmcc`, `lmcx`, `lmoc` for one prompt each; `gh api user`; `git fetch` in a repo; `clojure -Sdeps '{:deps {org.clojure/data.json {:mvn/version "2.5.0"}}}' -M -e '(println :ok)'`; `brew install hello`; `docker run --rm alpine true`. Note anything that fails and whether it is a missing proxy knob or a real block.

- [ ] **Step 6: Boot-window check**
  `limactl stop sandbox && limactl start sandbox`, then `limactl shell sandbox sudo -n true` still fails and `agent-egress-verify` still passes.

- [ ] **Step 7: After a week**
  Build the real allowlist from the decision log, remove `*`, push it with the `tee` one-liner, rerun the verify script and the agents. Then backlog any residual items (DNS fallback if resolved had to stay, pasta host-loopback address, containers needing more than the proxy).
