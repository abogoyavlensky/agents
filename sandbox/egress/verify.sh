#!/usr/bin/env bash
# agent-egress-verify - installed as /usr/local/bin/agent-egress-verify.
#
# Run as the `agent` user inside the sandbox VM after every recreation or
# reboot. Confirms that the egress controls are actually in force: no sudo, no
# direct network, no DNS, and a proxy that refuses anything off the allowlist.
set -u

PROXY="http://127.0.0.1:8080"
ALLOWLIST="/etc/agent-proxy/allowlist.txt"
DECISION_LOG="/var/log/agent-proxy/decisions.log"
SKIP=77

failures=0
detail=""

check() {
	local name="$1" fn="$2" rc=0
	detail=""
	# Capture the status here, not after an `if`: once the compound closes, $?
	# is the `if` statement's own status, not the check function's.
	"$fn" || rc=$?
	if [ "$rc" -eq 0 ]; then
		printf 'PASS  %s\n' "$name"
	elif [ "$rc" -eq "$SKIP" ]; then
		printf 'SKIP  %s%s\n' "$name" "${detail:+ - $detail}"
	else
		printf 'FAIL  %s%s\n' "$name" "${detail:+ - $detail}"
		failures=$((failures + 1))
	fi
	return 0
}

# True when the allowlist would allow $1: either it is listed verbatim or the
# file is in discovery mode. Deliberately simpler than the addon's matching -
# it only gates the checks below, it does not enforce anything.
allowlist_allows() {
	[ -r "$ALLOWLIST" ] || return 1
	sed -e 's/#.*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' "$ALLOWLIST" |
		grep -qxF -e '*' -e "$1"
}

# 1. The agent must not have general sudo.
check_no_sudo() {
	if sudo -n true >/dev/null 2>&1; then
		detail="sudo -n true succeeded"
		return 1
	fi
	return 0
}

# 2. ...but must keep the one command Lima's readiness probe runs on every start.
check_param_env_sudo() {
	if sudo -n cat /mnt/lima-cidata/param.env >/dev/null 2>&1; then
		return 0
	fi
	detail="sudo -n cat /mnt/lima-cidata/param.env failed; limactl start will hang"
	return 1
}

# 3. An allowlisted host is reachable through the proxy.
check_allowed_host() {
	local rc
	allowlist_allows example.com || {
		detail="example.com is not allowed by $ALLOWLIST"
		return "$SKIP"
	}
	curl -sS --max-time 5 -o /dev/null https://example.com >/dev/null 2>&1
	rc=$?
	[ "$rc" -eq 0 ] && return 0
	detail="curl exited $rc"
	return 1
}

# 4. A direct connection, bypassing the proxy, is refused by nftables. A numeric
#    address because the agent has no DNS: a hostname would fail at resolution
#    (exit 6) without ever exercising the firewall. A timeout (exit 28) means
#    packets are being dropped somewhere instead of rejected, which is a failure:
#    it would leave every blocked tool hanging.
check_direct_connection_refused() {
	local rc
	curl -sS --noproxy '*' --max-time 5 -o /dev/null https://1.1.1.1 >/dev/null 2>&1
	rc=$?
	case "$rc" in
	7) return 0 ;;
	28) detail="timed out (exit 28) instead of being rejected" ;;
	0) detail="direct connection to 1.1.1.1 succeeded" ;;
	*) detail="unexpected curl exit $rc" ;;
	esac
	return 1
}

# 5. The proxy refuses IP literals, so an allowlisted name is the only way out.
check_proxy_denies_ip_literal() {
	local out rc
	# --noproxy '' clears NO_PROXY, which lists 127.0.0.1: curl honours the
	# exclusion list even when -x is given, and would connect directly.
	out=$(curl -sS --max-time 5 --noproxy '' -o /dev/null -x "$PROXY" https://127.0.0.1/ 2>&1)
	rc=$?
	case "$out" in
	*403*) return 0 ;;
	esac
	detail="expected 403 from the proxy, got exit $rc: ${out:-no output}"
	return 1
}

# 6. The proxy refuses ports other than 443 (CONNECT) and 80 (plain HTTP).
check_proxy_denies_port() {
	local code
	code=$(curl -sS --max-time 5 -o /dev/null -w '%{http_code}' \
		-x "$PROXY" http://example.com:8443/ 2>/dev/null)
	[ "$code" = "403" ] && return 0
	detail="expected HTTP 403, got '${code:-nothing}'"
	return 1
}

# 7. The agent cannot resolve names at all: DNS is an exfiltration channel.
check_no_dns() {
	if getent hosts example.com >/dev/null 2>&1; then
		detail="getent resolved example.com"
		return 1
	fi
	return 0
}

# 8. Raw TCP to an arbitrary host and port is refused, and refused immediately.
check_raw_tcp_refused() {
	local start elapsed rc
	start=$SECONDS
	timeout 3 bash -c 'exec 3<>/dev/tcp/140.82.121.4/22' >/dev/null 2>&1
	rc=$?
	elapsed=$((SECONDS - start))
	if [ "$rc" -eq 0 ]; then
		detail="connected to 140.82.121.4:22"
		return 1
	fi
	if [ "$rc" -eq 124 ] || [ "$elapsed" -ge 3 ]; then
		detail="hung for ${elapsed}s instead of being rejected"
		return 1
	fi
	return 0
}

# 9. Decisions are recorded, so the allowlist can be built from real traffic.
check_decision_log() {
	allowlist_allows example.com || {
		detail="example.com is not allowed by $ALLOWLIST"
		return "$SKIP"
	}
	[ -r "$DECISION_LOG" ] || {
		detail="$DECISION_LOG is not readable by $(id -un)"
		return "$SKIP"
	}
	grep -q 'allow connect example\.com:443' "$DECISION_LOG" && return 0
	detail="no 'allow connect example.com:443' line in $DECISION_LOG"
	return 1
}

# 10. A ClientHello whose SNI differs from the CONNECT host is denied. This is
#     what stops an allowlisted, CDN-fronted host from being used to reach a
#     different site behind the same front.
check_sni_mismatch_denied() {
	local rc
	if ! allowlist_allows example.com || ! allowlist_allows example.org; then
		detail="example.com and example.org are not both allowed by $ALLOWLIST"
		return "$SKIP"
	fi
	curl -sS --max-time 5 -x "$PROXY" \
		--connect-to example.com:443:example.org:443 \
		-o /dev/null https://example.com >/dev/null 2>&1
	rc=$?
	if [ "$rc" -eq 0 ]; then
		detail="tunnel to example.org with SNI example.com succeeded"
		return 1
	fi
	if [ -r "$DECISION_LOG" ] && ! grep -q 'deny sni example\.org:443' "$DECISION_LOG"; then
		detail="handshake failed (exit $rc) but no 'deny sni example.org:443' line was logged"
		return 1
	fi
	return 0
}

printf 'agent-egress-verify: running as %s\n\n' "$(id -un)"

check "agent has no general sudo" check_no_sudo
check "agent keeps the Lima readiness sudo command" check_param_env_sudo
check "allowlisted host reachable through the proxy" check_allowed_host
check "direct connection refused by nftables" check_direct_connection_refused
check "proxy denies IP literals" check_proxy_denies_ip_literal
check "proxy denies non-standard ports" check_proxy_denies_port
check "agent cannot resolve DNS" check_no_dns
check "raw TCP refused immediately" check_raw_tcp_refused
check "decisions are logged" check_decision_log
check "SNI mismatch denied" check_sni_mismatch_denied

printf '\n'
if [ "$failures" -eq 0 ]; then
	printf 'all checks passed\n'
	exit 0
fi
printf '%d check(s) failed\n' "$failures"
exit 1
