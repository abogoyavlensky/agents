#!/usr/bin/env bash
# Acceptance tests for the transparent-Squid egress spike. Run inside the VM as
# `agent`, from the Mac:
#
#   limactl shell <vm> bash -s < sandbox/spike/verify.sh
#
# ALLOWED must be on the allowlist and DENIED must not be; the defaults fit the
# built-in list. Exits non-zero if any check fails.
set -u
ALLOWED="${ALLOWED:-github.com}"
DENIED="${DENIED:-example.com}"
SSH_ON_443="${SSH_ON_443:-ssh.github.com}" # a real SSH server on port 443, under ALLOWED
LIMA_DNS="${LIMA_DNS:-192.168.5.3}"
SINCE="$(date '+%Y-%m-%d %H:%M:%S')"
fails=0

check() { # name, function
	if "$2" >/dev/null 2>&1; then printf 'PASS  %s\n' "$1"
	else printf 'FAIL  %s\n' "$1"; fails=$((fails + 1)); fi
}
skip() { printf 'SKIP  %s - %s\n' "$1" "$2"; }
# Prints the HTTP status, or "curl:<exit code>" when no response arrived.
http_code() {
	local code rc
	code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 15 "$@" 2>/dev/null)"; rc=$?
	[ "$rc" -eq 0 ] && echo "$code" || echo "curl:$rc"
}
squid_log() { journalctl -t squid -o cat --since "$SINCE" 2>/dev/null; }
# A TLS connection is blocked when Squid never spliced it: the client gets only
# an error page (403/409 on Squid's own certificate) or a dead connection, and
# the journal shows no TCP_TUNNEL/200 for that target.
tls_blocked() { # sni-or-ip, connect-ip
	local code since
	sleep 1; since="$(date '+%Y-%m-%d %H:%M:%S')" # journal has 1s granularity
	code="$(http_code -k --resolve "$1:443:$2" "https://$1/")"
	case "$code" in 403 | 409 | curl:*) ;; *) return 1 ;; esac
	! journalctl -t squid -o cat --since "$since" 2>/dev/null | grep -qE "TCP_TUNNEL/200 CONNECT $1:443"
}
refused_fast() { # host, port: connection refused, not a hang
	timeout 3 bash -c "exec 3<>/dev/tcp/$1/$2" 2>/dev/null
	[ $? -eq 1 ]
}

echo "== privileges"
no_sudo() { ! sudo -n true; }
readiness_sudo() { sudo -n cat /mnt/lima-cidata/param.env; }
check "agent has no general sudo" no_sudo
check "Lima readiness sudo still works" readiness_sudo

echo "== allowed traffic"
https_allowed() { curl -fsS -o /dev/null --max-time 15 "https://$ALLOWED/"; }
https_subdomain() { curl -fsS -o /dev/null --max-time 15 "https://api.$ALLOWED/"; }
http_allowed() { case "$(http_code "http://$ALLOWED/")" in 2?? | 3??) ;; *) return 1 ;; esac; }
tunnel_logged() { squid_log | grep -qE "TCP_TUNNEL/200 CONNECT $ALLOWED:443"; }
check "HTTPS to $ALLOWED" https_allowed
check "HTTPS to a subdomain (api.$ALLOWED)" https_subdomain
check "plain HTTP to $ALLOWED answers" http_allowed
check "spliced tunnel logged in the journal" tunnel_logged
IP="$(getent ahostsv4 "$ALLOWED" 2>/dev/null | awk 'NR == 1 { print $1 }')"
[ -n "$IP" ] || { echo "cannot resolve $ALLOWED; aborting"; exit 1; }

echo "== denied traffic"
denied_nxdomain() { ! getent hosts "$DENIED"; }
denied_curl6() { [ "$(http_code "https://$DENIED/")" = curl:6 ]; }
cdn_hole() { tls_blocked "$DENIED" "$IP"; }
reverse_hole() { tls_blocked "$ALLOWED" 1.1.1.1; }
reverse_hole_http() { [ "$(http_code --resolve "$ALLOWED:80:1.1.1.1" "http://$ALLOWED/")" = 409 ]; }
no_sni() { tls_blocked "$IP" "$IP"; }
# ssh.github.com speaks SSH on 443. Send an SSH banner through an allowed name:
# if the server's banner comes back, non-TLS bytes were tunnelled.
ssh_on_443_blocked() {
	local banner
	banner="$(timeout 5 bash -c "exec 3<>/dev/tcp/$SSH_ON_443/443; printf 'SSH-2.0-probe\r\n' >&3; head -c 4 <&3" 2>/dev/null)"
	[ "$banner" != "SSH-" ]
}
port22() { refused_fast "$IP" 22; }
port8443() { refused_fast "$IP" 8443; }
lima_dns() { refused_fast "$LIMA_DNS" 53; }
squid_fwd_direct() { refused_fast 127.0.0.1 3128; }
squid_http_direct() { refused_fast 127.0.0.1 3129; }
squid_tls_direct() { refused_fast 127.0.0.1 3130; }
check "$DENIED does not resolve" denied_nxdomain
check "HTTPS to $DENIED fails at DNS (curl 6)" denied_curl6
check "CDN hole: $ALLOWED's IP with SNI $DENIED is refused" cdn_hole
check "reverse hole: SNI $ALLOWED sent to 1.1.1.1 is refused" reverse_hole
check "reverse hole over HTTP: Host $ALLOWED at 1.1.1.1 gets 409" reverse_hole_http
check "no SNI: raw https://$IP/ is refused" no_sni
check "SSH on 443 ($SSH_ON_443) is not tunnelled" ssh_on_443_blocked
check "port 22 on $ALLOWED's IP is refused fast" port22
check "port 8443 on $ALLOWED's IP is refused fast" port8443
check "direct DNS to the Lima resolver is refused" lima_dns
check "Squid's forward-proxy port is unreachable directly" squid_fwd_direct
check "Squid's HTTP listener is unreachable directly" squid_http_direct
check "Squid's TLS listener is unreachable directly" squid_tls_direct

echo "== containers"
container_allowed() { docker run --rm curlimages/curl:latest -fsS -o /dev/null --max-time 20 "https://api.$ALLOWED/"; }
container_denied() { ! docker run --rm curlimages/curl:latest -sS -o /dev/null --max-time 20 "https://$DENIED/"; }
if command -v docker >/dev/null && docker info >/dev/null 2>&1; then
	check "container reaches api.$ALLOWED" container_allowed
	check "container cannot reach $DENIED" container_denied
else
	skip "container checks" "docker not available"
fi

echo "== operations"
journal_readable() { journalctl -t squid -n 1; }
reload_from_host() { sudo -n agent-egress-reload | grep -q 'from /mnt/lima-sandbox/allowlist.txt'; }
reload_runs() { sudo -n agent-egress-reload; }
check "agent can read Squid's decisions" journal_readable
if [ -r /mnt/lima-sandbox/allowlist.txt ]; then
	check "active policy comes from the host file" reload_from_host
else
	skip "host-managed allowlist" "no ~/.config/lima-sandbox/allowlist.txt on the Mac"
	check "reload runs from agent" reload_runs
fi

echo
echo "IP-belongs-to-name rejections since boot (should stay near zero):"
echo "  $(journalctl -t squid -b -o cat 2>/dev/null | grep -c '/409 ') x 409"
if [ "$fails" -eq 0 ]; then echo "all checks passed"; else echo "$fails check(s) failed"; fi
exit "$fails"
