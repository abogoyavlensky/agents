#!/usr/bin/env python3
"""Build a prefilled GitHub "new issue" URL.

GitHub's /issues/new endpoint accepts query params that prefill the issue form:
title, body, labels, assignees, milestone, and template. Opening the resulting
URL in a browser (where the user is authed as themselves) lands them on the new
-issue page with everything filled in — they just review and click "Submit".

This is the reliable way to help a user file an issue on a repo the API can't
touch: fine-grained PATs / GITHUB_TOKEN can only create issues on repos they're
authorized for, but the web UI lets any logged-in user file on any public repo.

Usage:
  issue_url.py --repo owner/name --title "..." --body-file body.md
  issue_url.py --repo owner/name --title "..." --body "inline text" \
               --labels bug,help-wanted --assignees octocat

Prints the URL to stdout. Diagnostics (length, warnings) go to stderr so the
URL stays clean for piping.
"""
import argparse
import sys
import urllib.parse

# GitHub rejects long prefill URLs with a 400, and the ceiling is far lower than
# the HTTP limit — a ~4 KB URL has been observed to fail. Warn well below that so
# the body gets trimmed before it breaks; ~2 KB is reliable in practice.
SAFE_URL_LEN = 2000


def main():
    p = argparse.ArgumentParser(description="Build a prefilled GitHub new-issue URL.")
    p.add_argument("--repo", required=True, help="Target repo as owner/name.")
    p.add_argument("--title", required=True, help="Issue title.")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--body", help="Issue body (markdown) as an inline string.")
    g.add_argument("--body-file", help="Path to a file containing the body.")
    p.add_argument("--labels", help="Comma-separated label names.")
    p.add_argument("--assignees", help="Comma-separated GitHub usernames.")
    p.add_argument("--milestone", help="Milestone name.")
    p.add_argument(
        "--template",
        help="Issue-template filename (e.g. bug_report.md). Note: when a "
        "template is set, GitHub ignores --body in favor of the template.",
    )
    args = p.parse_args()

    if "/" not in args.repo:
        p.error("--repo must be owner/name")

    body = ""
    if args.body_file:
        with open(args.body_file, encoding="utf-8") as f:
            body = f.read()
    elif args.body:
        body = args.body

    # Only include params that are set, so we don't stuff empty keys into the URL.
    params = {"title": args.title}
    if body:
        params["body"] = body
    if args.labels:
        params["labels"] = args.labels
    if args.assignees:
        params["assignees"] = args.assignees
    if args.milestone:
        params["milestone"] = args.milestone
    if args.template:
        params["template"] = args.template

    query = urllib.parse.urlencode(params)
    url = f"https://github.com/{args.repo}/issues/new?{query}"

    if args.template and body:
        print(
            "note: --template is set, so GitHub will show the template and "
            "ignore --body.",
            file=sys.stderr,
        )
    if len(url) > SAFE_URL_LEN:
        print(
            f"warning: URL is {len(url)} chars (> {SAFE_URL_LEN}). GitHub may "
            "reject a long prefill URL with a 400. Trim the body (keep the full "
            "version in the saved file to paste after filing), or use "
            "`gh issue create --body-file` if you have write access.",
            file=sys.stderr,
        )
    else:
        print(f"url length: {len(url)} chars", file=sys.stderr)

    print(url)


if __name__ == "__main__":
    main()
