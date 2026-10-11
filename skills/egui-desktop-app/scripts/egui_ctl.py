#!/usr/bin/env python3
"""Talk to an egui app's JSON-lines control server (see the egui-desktop-app skill).

  egui_ctl.py [--port N] [--wait SECONDS] METHOD [PARAMS_JSON]
  egui_ctl.py [--port N] [--wait SECONDS] --batch FILE   # one {"method", "params"} per line ("-" = stdin)

Prints each reply as one JSON line. Exit status: 0 all ok, 1 a reply had "ok": false,
2 could not connect. `--wait` retries the connection while the app starts.
The port defaults to $EGUI_CONTROL_PORT, then 7979.
"""
import argparse
import json
import os
import socket
import sys
import time


def connect(port: int, wait: float) -> socket.socket:
    deadline = time.monotonic() + wait
    while True:
        try:
            return socket.create_connection(("127.0.0.1", port), timeout=60)
        except OSError as e:
            if time.monotonic() >= deadline:
                print(f"egui_ctl: cannot connect to 127.0.0.1:{port}: {e}", file=sys.stderr)
                sys.exit(2)
            time.sleep(0.25)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", type=int, default=int(os.environ.get("EGUI_CONTROL_PORT", "7979")))
    ap.add_argument("--wait", type=float, default=0.0, help="seconds to keep retrying the connection")
    ap.add_argument("--batch", help="file of JSON requests, one per line ('-' for stdin)")
    ap.add_argument("method", nargs="?")
    ap.add_argument("params", nargs="?", default="{}")
    a = ap.parse_args()

    if a.batch:
        src = sys.stdin if a.batch == "-" else open(a.batch)
        requests = [json.loads(l) for l in src if l.strip() and not l.lstrip().startswith("#")]
    elif a.method:
        requests = [{"method": a.method, "params": json.loads(a.params)}]
    else:
        ap.error("give METHOD [PARAMS_JSON] or --batch FILE")

    sock = connect(a.port, a.wait)
    stream = sock.makefile("rw", encoding="utf-8")
    failed = False
    for i, req in enumerate(requests, 1):
        req = {"id": i, "method": req["method"], "params": req.get("params", {})}
        stream.write(json.dumps(req) + "\n")
        stream.flush()
        line = stream.readline()
        if not line:
            print("egui_ctl: connection closed by the app", file=sys.stderr)
            sys.exit(2)
        print(line.rstrip("\n"))
        if not json.loads(line).get("ok", False):
            failed = True
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
