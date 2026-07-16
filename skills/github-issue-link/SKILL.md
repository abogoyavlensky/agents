---
name: github-issue-link
description: >-
  File a GitHub issue by generating a prefilled one-click "new issue" link
  (github.com/owner/repo/issues/new?title=...&body=...) that opens the issue
  form with the title, body, and labels already filled in. Use this whenever
  the user wants to open, file, create, or report a GitHub issue or bug — and
  especially when `gh issue create` fails with a permissions error, when the
  target repo is one the user does not own (an upstream/third-party repo, a
  cross-fork bug report), or when the user wants to review the issue in the
  browser before submitting or wants a shareable link to file it. Also use it
  as the fallback the moment an API-based issue creation is rejected.
---

# GitHub issue one-click link

Filing an issue through the API (`gh issue create`) needs a token authorized
for that repo. Fine-grained PATs and `GITHUB_TOKEN` are typically scoped to the
user's **own** repos, so creating an issue on an upstream or third-party repo
fails with `Resource not accessible by personal access token`. But GitHub's web
UI lets **any logged-in user** file an issue on any public repo — so a prefilled
`issues/new` link that the user opens in their browser sidesteps the whole
permission problem. It also gives the user a chance to eyeball the issue before
it goes public.

This skill builds that link.

## When to use a link vs. `gh issue create`

Reach for `gh issue create --title ... --body-file ...` when **all** of these
hold: the repo is the user's own (or one they have write access to), and they
haven't asked to review it first. It's the least-friction path when it works.

Otherwise — the repo isn't theirs, `gh` returned a permission error, or the
user wants to review/share before submitting — build a prefill link. When in
doubt, a link is the safe default: it never silently fails, and the user stays
in control of the final submit.

## Workflow

1. **Pin the target repo** (`owner/name`). Take it from what the user said, or
   infer it from git remotes — for a bug report against an upstream project,
   prefer the `upstream` remote over `origin` (which is usually the user's
   fork). `gh repo view --json nameWithOwner -q .nameWithOwner` and
   `git remote -v` both help. If it's genuinely ambiguous which repo the issue
   belongs on, ask rather than guess — filing on the wrong repo is public and
   awkward to undo.

2. **Draft the title and body**, then **write the body to a file** (e.g. a
   scratch/temp path). Saving it means the user has a copy-paste fallback if the
   link ever misbehaves, and you can regenerate the URL without retyping. Write
   the body as normal GitHub-flavored markdown — the script handles all
   URL-encoding, so don't pre-encode anything.

   Keep the body about the **problem or proposal itself** — what's missing or
   broken and why it matters (and, for a feature, the shape of the change) — so
   it stands on its own for anyone reading it later. Don't tie it to a specific
   upcoming PR ("a branch is ready", "I'll open a PR referencing this"): an
   issue outlives any one PR, maintainers usually want to discuss the design
   before code lands, and such promises read as presumptuous and go stale if the
   PR changes or never appears. Describe the need and let the PR's own
   description carry the implementation — if a fix already exists, note that a
   solution looks feasible, not that a particular branch/PR is inbound.

3. **Build the URL** with the bundled script:

   ```bash
   python3 <skill-dir>/scripts/issue_url.py \
     --repo owner/name \
     --title "Concise, specific title" \
     --body-file /path/to/body.md
   ```

   Optional flags: `--labels bug,help-wanted`, `--assignees user1,user2`,
   `--milestone "v2"`, `--template bug_report.md`. (With `--template`, GitHub
   shows the repo's template and ignores `--body`.) The script prints the URL to
   stdout and a length note to stderr.

4. **Give the user the link.** Present the raw URL on its own — the terminal
   renders it clickable. Say they'll be authed as themselves in the browser with
   the form prefilled, so they just review and submit. Mention the body is also
   saved at `<path>` for copy-paste.

5. **Mind the length.** GitHub rejects long prefill URLs with a **400**, and the
   ceiling is far lower than the HTTP limit — a ~4 KB URL has been observed to
   400 — so keep it well under ~2 KB (the script warns past that). If the body is
   too big, trim it to the essentials and keep the fuller version in the saved
   file for the user to paste in after filing (GitHub lets them edit the body).
   If the user has write access, `gh issue create --body-file` sidesteps the URL
   limit entirely.

## After filing

Once the user has filed and has the issue number, `gh issue edit` **does** work
on issues (even ones filed via the browser) if the user has access, so you can
follow up: cross-link related issues, add a reference, or tweak wording.

## Example

**User:** "Can you file a bug on nooga/let-go about `ns-publics` being missing?"

1. Confirm/pin repo: `nooga/let-go` (an upstream repo the user's PAT can't post
   to — a link is the right call).
2. Draft the body, write it to `.tmp/issue-ns-publics.md`.
3. Run:

   ```bash
   python3 <skill-dir>/scripts/issue_url.py \
     --repo nooga/let-go \
     --title "Missing ns-publics (and ns-interns/ns-map)" \
     --body-file .tmp/issue-ns-publics.md \
     --labels bug
   ```

4. Hand back the printed `https://github.com/nooga/let-go/issues/new?...` link,
   noting the body is saved at `.tmp/issue-ns-publics.md`.
