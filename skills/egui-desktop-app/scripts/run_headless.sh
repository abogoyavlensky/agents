#!/usr/bin/env bash
# Start an egui/eframe app for an agent to drive, in the background, and wait for its control port.
#
#   run_headless.sh [--port N] [--log FILE] [--timeout SECONDS] -- COMMAND [ARGS...]
#
# Linux without a display ($DISPLAY and $WAYLAND_DISPLAY unset): runs under Xvfb (xvfb-run), and
# wgpu falls back to the software Vulkan driver (Mesa lavapipe) when there is no GPU.
# macOS / Windows / Linux desktops: runs the command as is (the window opens on screen).
#
# Prints "pid=<pid> log=<file>" once the port accepts connections (or right away without --port).
# Stop the app with `egui_ctl.py app.quit` or `kill <pid>`.
set -euo pipefail

port="" log="" timeout=120
while [[ $# -gt 0 ]]; do
  case "$1" in
    --port) port="$2"; shift 2 ;;
    --log) log="$2"; shift 2 ;;
    --timeout) timeout="$2"; shift 2 ;;
    --) shift; break ;;
    *) break ;;
  esac
done
[[ $# -gt 0 ]] || { echo "usage: $0 [--port N] [--log FILE] [--timeout S] -- COMMAND [ARGS...]" >&2; exit 2; }
log="${log:-$(mktemp -t egui-app.XXXXXX.log)}"

cmd=("$@")
if [[ "$(uname -s)" == "Linux" && -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  command -v xvfb-run >/dev/null || { echo "no display and no xvfb-run: install xvfb (apt install xvfb)" >&2; exit 2; }
  cmd=(xvfb-run -a -s "-screen 0 1920x1200x24" "${cmd[@]}")
fi

"${cmd[@]}" >"$log" 2>&1 &
pid=$!

if [[ -n "$port" ]]; then
  for ((i = 0; i < timeout * 4; i++)); do
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "app exited before opening port $port; log $log:" >&2
      tail -n 30 "$log" >&2
      exit 1
    fi
    if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
      echo "pid=$pid log=$log"
      exit 0
    fi
    sleep 0.25
  done
  echo "port $port not open after ${timeout}s; log $log:" >&2
  tail -n 30 "$log" >&2
  kill "$pid" 2>/dev/null || true
  exit 1
fi
echo "pid=$pid log=$log"
