# Installed as /etc/profile.d/agent-proxy.sh by the sandbox egress provisioning.
# POSIX sh: profile.d scripts are sourced by dash on Ubuntu.
#
# Points every non-root tool at the local allowlisting proxy. Root is skipped on
# purpose: provisioning runs as root and must reach the network before the proxy
# exists. profile.d rather than Lima's env: for the same reason - Lima writes
# env: to /etc/environment, which root's bootstrap steps would inherit.

if [ "$(id -u)" != "0" ]; then
	HTTP_PROXY="http://127.0.0.1:8080"
	HTTPS_PROXY="http://127.0.0.1:8080"
	# Lowercase variants too: curl deliberately ignores uppercase HTTP_PROXY.
	http_proxy="http://127.0.0.1:8080"
	https_proxy="http://127.0.0.1:8080"
	NO_PROXY="localhost,127.0.0.1,::1"
	no_proxy="localhost,127.0.0.1,::1"
	export HTTP_PROXY HTTPS_PROXY http_proxy https_proxy NO_PROXY no_proxy

	# Node 24+ only honours the proxy variables when this is set.
	NODE_USE_ENV_PROXY=1
	export NODE_USE_ENV_PROXY

	# The JVM ignores the proxy variables. This prints one "Picked up
	# JAVA_TOOL_OPTIONS" line to stderr per launch; comment the two lines below
	# out if that noise is worse than the convenience.
	JAVA_TOOL_OPTIONS="-Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=8080 -Dhttps.proxyHost=127.0.0.1 -Dhttps.proxyPort=8080 -Dhttp.nonProxyHosts=localhost|127.0.0.1"
	export JAVA_TOOL_OPTIONS
fi
