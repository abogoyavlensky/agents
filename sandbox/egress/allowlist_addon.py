"""mitmproxy addon: allow only allowlisted hostnames out of the sandbox VM.

Enforces policy without intercepting TLS. CONNECT requests are answered with a
403 unless the target host is on the allowlist, plain HTTP is judged the same
way, and the TLS ClientHello's SNI must match the host the client asked to
connect to - otherwise interception is left on and the handshake fails, because
nothing in the VM trusts mitmproxy's CA.

Every hook sets the deny outcome before doing anything that can raise, so a bug
in here refuses traffic rather than passing it.

The mitmproxy imports are guarded so that `parse_rules` and `decide` can be
unit-tested on a machine without mitmproxy installed.
"""

import datetime
import ipaddress
import logging
import os

try:
    from mitmproxy import ctx, http
except ImportError:  # unit tests, or any machine without mitmproxy
    ctx = None
    http = None

logger = logging.getLogger(__name__)

DEFAULT_ALLOWLIST = "/etc/agent-proxy/allowlist.txt"
DEFAULT_DECISION_LOG = "/var/log/agent-proxy/decisions.log"

# Names that must never leave the VM, whatever the allowlist says: they resolve
# to the VM itself, the Mac, or something else on the local network.
LOCAL_SUFFIXES = (".localhost", ".local", ".internal")

# Port allowed per kind of request. HTTPS is tunnelled through CONNECT, plain
# HTTP is proxied directly; nothing else gets through.
PORT_BY_KIND = {"connect": 443, "http": 80}

BLOCKED_BODY = b"blocked by agent-proxy allowlist\n"


class Rules:
    """A parsed allowlist: a set of domains, or "everything" in discovery mode."""

    __slots__ = ("allow_all", "domains")

    def __init__(self, allow_all, domains):
        self.allow_all = allow_all
        self.domains = frozenset(domains)

    def __repr__(self):
        return "Rules(allow_all=%r, domains=%r)" % (self.allow_all, sorted(self.domains))


def _normalize_host(host):
    """Lowercase a host and strip brackets, a trailing root dot and whitespace."""
    value = (host or "").strip().lower()
    if value.startswith("[") and value.endswith("]"):
        value = value[1:-1]
    if len(value) > 1 and value.endswith("."):
        value = value[:-1]
    return value


def parse_rules(text):
    """Parse allowlist text into Rules. One entry per line, `#` starts a comment.

    An entry matches itself and all its subdomains. A line containing only `*`
    turns on discovery mode, allowing every hostname.
    """
    allow_all = False
    domains = set()
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line == "*":
            allow_all = True
            continue
        # `*.example.com` and `.example.com` are common spellings of the same
        # intent as `example.com`, which already covers subdomains.
        if line.startswith("*."):
            line = line[1:]
        entry = _normalize_host(line.lstrip("."))
        if entry:
            domains.add(entry)
    return Rules(allow_all, domains)


def decide(host, port, kind, rules):
    """Decide whether one request may proceed.

    `kind` is "connect" (HTTPS via CONNECT) or "http" (plain proxied HTTP).
    Returns (allowed, reason); every denial has its own reason string.
    """
    if rules is None:
        return False, "no-allowlist"

    name = _normalize_host(host)
    if not name:
        return False, "no-host"

    expected_port = PORT_BY_KIND.get(kind)
    if expected_port is None:
        return False, "bad-kind"

    # IP literals bypass name-based policy entirely, so they are never allowed -
    # not even in discovery mode.
    try:
        ipaddress.ip_address(name)
    except ValueError:
        pass
    else:
        return False, "ip-literal"

    if name == "localhost" or name.endswith(LOCAL_SUFFIXES):
        return False, "local-name"

    if port != expected_port:
        return False, "bad-port"

    if rules.allow_all:
        return True, "allow-all"

    for domain in rules.domains:
        if name == domain or name.endswith("." + domain):
            return True, "allowlist"

    return False, "not-in-allowlist"


class AllowlistAddon:
    def __init__(self):
        self._rules_cache = None
        self._stamp = None

    # -- options ---------------------------------------------------------

    def load(self, loader):
        loader.add_option(
            "allowlist", str, DEFAULT_ALLOWLIST, "Path to the hostname allowlist."
        )
        loader.add_option(
            "decision_log", str, DEFAULT_DECISION_LOG, "Path to the decision log."
        )

    def configure(self, updates):
        if "allowlist" in updates:
            self._stamp = None  # force a reload on the next decision

    # -- allowlist -------------------------------------------------------

    def _rules(self):
        """Current rules, reloaded when the allowlist file changed. None = deny all."""
        path = getattr(ctx.options, "allowlist", DEFAULT_ALLOWLIST) if ctx else None
        try:
            stat = os.stat(path)
            stamp = (stat.st_mtime_ns, stat.st_size)
        except OSError as exc:
            if self._stamp is not None or self._rules_cache is not None:
                logger.warning("agent-proxy: allowlist %s unreadable (%s)", path, exc)
            self._rules_cache = None
            self._stamp = None
            return None

        if stamp != self._stamp:
            try:
                with open(path, "r", encoding="utf-8") as handle:
                    self._rules_cache = parse_rules(handle.read())
            except Exception as exc:  # unreadable or undecodable: deny everything
                logger.warning("agent-proxy: cannot parse allowlist %s (%s)", path, exc)
                self._rules_cache = None
            self._stamp = stamp
        return self._rules_cache

    # -- decision log ----------------------------------------------------

    def _log(self, allowed, kind, host, port, reason):
        line = "%s %s %s %s:%s %s\n" % (
            datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
            "allow" if allowed else "deny",
            kind,
            host or "-",
            port,
            reason,
        )
        path = getattr(ctx.options, "decision_log", DEFAULT_DECISION_LOG) if ctx else None
        try:
            with open(path, "a", encoding="utf-8") as handle:
                handle.write(line)
                handle.flush()
        except OSError as exc:
            logger.warning("agent-proxy: cannot write decision log %s (%s)", path, exc)

    # -- hooks -----------------------------------------------------------

    def http_connect(self, flow):
        # Deny first: if anything below raises, the 403 stands.
        flow.response = http.Response.make(
            403, BLOCKED_BODY, {"Content-Type": "text/plain"}
        )
        host, port, allowed, reason = "", 0, False, "internal-error"
        try:
            host = flow.request.pretty_host
            port = flow.request.port
            allowed, reason = decide(host, port, "connect", self._rules())
            if allowed:
                flow.response = None
        except Exception as exc:
            logger.warning("agent-proxy: http_connect failed (%s)", exc)
        self._log(allowed, "connect", host, port, reason)

    def request(self, flow):
        # Plain proxied HTTP, and any request inside a tunnel we ended up
        # intercepting - the port rule denies the latter.
        if flow.response is not None:
            return
        flow.response = http.Response.make(
            403, BLOCKED_BODY, {"Content-Type": "text/plain"}
        )
        host, port, allowed, reason = "", 0, False, "internal-error"
        try:
            host = flow.request.pretty_host
            port = flow.request.port
            allowed, reason = decide(host, port, "http", self._rules())
            if allowed:
                flow.response = None
        except Exception as exc:
            logger.warning("agent-proxy: request failed (%s)", exc)
        self._log(allowed, "http", host, port, reason)

    def tls_clienthello(self, data):
        # Leaving ignore_connection false means mitmproxy intercepts, and since
        # no client here trusts its CA the handshake fails - which is the denial.
        host, port, sni = "", 0, ""
        try:
            address = data.context.server.address
            if address:
                host = _normalize_host(address[0])
                port = address[1]
            sni = _normalize_host(data.client_hello.sni or "")
            if host and sni and sni == host:
                data.ignore_connection = True
                self._log(True, "sni", host, port, "sni-match")
                return
            reason = "sni-missing" if not sni else "sni-mismatch:%s" % sni
        except Exception as exc:
            logger.warning("agent-proxy: tls_clienthello failed (%s)", exc)
            reason = "internal-error"
        self._log(False, "sni", host, port, reason)


if ctx is not None:
    addons = [AllowlistAddon()]
