---
name: lgx-homebrew-release
description: Set up a Homebrew (and mise) release pipeline for a let-go CLI project built with lgx — GitHub Actions that build cross-platform binaries, publish a versioned GitHub Release, and update a Homebrew tap formula so users can `brew install <owner>/tap/<name>`. Use this whenever the user wants to release, publish, distribute, or ship an lgx/let-go CLI; add `brew install` support or a Homebrew formula; set up a release GitHub Actions workflow; make an lgx tool installable; or cut versioned releases — even if they don't say "Homebrew" explicitly. Applies to projects with an `lgx.edn` and a `:targets {:bin ...}` output (e.g. wtr, lgx, skl).
---

# Add a Homebrew release pipeline to an lgx project

## What this sets up

A tag-driven release pipeline, matching the wtr/lgx/skl convention:

1. You bump `resources/VERSION` and run `lgx release` — it tags `vX.Y.Z` and pushes.
2. The pushed tag triggers `.github/workflows/release.yml`, which:
   - builds the binary for **four targets** (linux/darwin × amd64/arm64) by
     bundling each platform's let-go base (`lgx build -bundle-base …`),
   - packages `NAME_<version>_<target>.tar.gz` + `checksums.txt`,
   - publishes a **GitHub Release** with those assets,
   - regenerates the **Homebrew formula** and pushes it to the user's tap.
3. Users then install with `brew install <owner>/tap/<name>`. Because the
   release publishes standard GitHub assets, `mise use -g github:<owner>/<repo>`
   works too, for free.

The version lives in **one place** — `resources/VERSION` — which `main.lg`
reads for `--version`, `tag.lg` reads to build the tag, and the formula test
asserts against. One file to bump per release.

## Prerequisite check

This skill only fits a let-go CLI built with **lgx**. Confirm the target repo has:
- an `lgx.edn` with a `:targets {:bin {:out "bin/<name>"}}` entry,
- a `.mise.toml` pinning `lg` and `lgx` under `[tools]`,
- a git remote (or the user can tell you the `owner/repo`).

If it is not an lgx project (no `lgx.edn`), stop and say so — this pipeline is
specific to lgx's build tooling.

## Step 1 — Resolve the project values (then confirm)

Derive these from the repo rather than guessing, then show the user the
resolved table and ask them to confirm or correct before writing anything:

- **name** — the binary, from `lgx.edn` `:targets {:bin {:out "bin/NAME"}}`.
- **owner / repo** — from `git remote get-url origin`
  (`git@github.com:OWNER/REPO.git` or `https://github.com/OWNER/REPO`). If there
  is no remote, ask the user.
- **desc** — a one-line formula description. Start from `lgx.edn` `:doc`, but
  trim it to Homebrew style: no leading article ("A"/"The"), no trailing
  period, ≤ ~80 chars. Confirm with the user.
- **tap** — `<owner>/homebrew-tap` by convention (this is what
  `brew install <owner>/tap/<name>` resolves to). Only change it if the user
  keeps their tap elsewhere.
- **lg version** — the `lg` pin under `[tools]` in `.mise.toml` (for
  `:lg-version` in `lgx.edn`).

Print them back like:

```
name:     gizmo        owner/repo: acme/gizmo
desc:     Do the thing across your repos
tap:      acme/homebrew-tap  ->  brew install acme/tap/gizmo
homepage: https://github.com/acme/gizmo
lg:       1.11.1
```

## Step 2 — Scaffold the release files

Run the renderer (path is relative to **this skill's directory**). It writes
the four new files + `resources/VERSION`, substituting the project identity,
and **skips any file that already exists** so it never clobbers existing CI:

```bash
python3 assets/render.py \
  --target <repo-root> \
  --name <name> --owner <owner> --repo <repo> \
  --desc "<desc>" --version <current-version>
```

It creates:
- `scripts/generate-formula.sh` — renders the formula from a release
  `checksums.txt` (executable).
- `scripts/tag.lg` — tags `v<VERSION>` from `resources/VERSION`.
- `.github/workflows/checks.yml` — fmt + tests, reusable via `workflow_call`.
- `.github/workflows/release.yml` — the build/release/homebrew jobs.
- `resources/VERSION` — the version, **with no trailing newline** (see gotchas).

**If a file was skipped because it already exists**, don't ignore it — open it
and reconcile by hand. The two common cases:
- An existing `checks.yml` is usually fine to keep as-is (the release just needs
  *a* reusable `checks` workflow to call).
- An existing `release.yml` likely needs the `homebrew` job appended and the
  smoke test pointed at a real command (see gotchas). Diff it against
  `assets/templates/release.yml` and merge the missing pieces.

If the project already has an `lgx lint` task, add a `Run lint` step to
`checks.yml` (between fmt and tests) — it keeps releases from shipping
lint-broken code. Leave it out if there's no lint task.

## Step 3 — Single-source the version

Two edits to existing files (the renderer intentionally leaves these alone):

**`main.lg`** — read the version from the resource instead of a hardcoded
string, so there's one source of truth. Add `[io :as io]` to the `ns` requires
(keep them sorted) and:

```clojure
(defn- version []
  (io/slurp (io/resource "VERSION")))
```

then set `:version (version)` in the app spec. If `main.lg` already reads
`VERSION` this way (e.g. copied from wtr), leave it.

**`lgx.edn`** — add, if missing:
- `:resource-paths ["resources"]` (so `VERSION` is on the path and bundled),
- `:lg-version "<lg from .mise.toml>"`,
- a `release` task:

```clojure
release {:doc "Release a new version"
         :extra-paths ["scripts"]
         :do [{:run "scripts/tag.lg"}
              {:sh "git push --tags"}]}
```

## Step 4 — Verify locally

Prove it works before handing back. Run from the repo root:

```bash
lgx build && ./bin/<name> --version          # prints "<name> <version>" from the resource
lgx check                                     # fmt + lint + tests still green
python3 -c "import yaml;[yaml.safe_load(open(f)) for f in ['.github/workflows/checks.yml','.github/workflows/release.yml']]" && echo "yaml ok"
```

Smoke-test the formula generator with a synthetic checksums file so you don't
need a real release:

```bash
v=<current-version>
for t in darwin_amd64 darwin_arm64 linux_amd64 linux_arm64; do
  printf '%064d  %s_%s_%s.tar.gz\n' 0 <name> "$v" "$t"
done > /tmp/ck.txt
bash scripts/generate-formula.sh "$v" /tmp/ck.txt   # should print a valid formula
```

Do **not** run `lgx release` or `scripts/tag.lg` yourself — that creates and
pushes a real tag, triggering a live release. Leave that to the user.

## Step 5 — README + the manual prerequisites

Add an **Installation** section to the README (Homebrew / mise / manual) if one
isn't there. See `references/manual-setup.md` for the exact snippets, plus the
two things the user must do on GitHub that you cannot:
- create the `<owner>/homebrew-tap` repo (once, if it doesn't exist),
- add a `HOMEBREW_TAP_TOKEN` secret (a PAT with write access to the tap).

Read `references/manual-setup.md` and relay those steps, and how to cut the
first release (`bump resources/VERSION` → commit → `lgx release`).

## Key details and gotchas

These are the things that quietly break a release if you miss them:

- **`resources/VERSION` must have no trailing newline.** `tag.lg` builds the
  tag as `"v" + <contents>`, so a stray `\n` yields a broken tag like
  `v0.1.0\n`. The renderer strips it; if you hand-edit, use
  `printf '0.1.0' > resources/VERSION` and verify with `wc -c`.
- **The release smoke test must call a command the CLI actually has.** The
  template uses `<name> --version` (universal). If you adapt an existing
  workflow that smoke-tests a subcommand (e.g. `wtr list`), make sure that
  subcommand exists — otherwise the release job fails.
- **The Homebrew formula class is the CamelCase of the name** (`skl` → `Skl`,
  `my-tool` → `MyTool`). The renderer computes this; keep it in sync if you edit.
- **The lg version is read from `.mise.toml`, not `lgx.edn`.** The release
  workflow greps `[tools] lg = "…"` to fetch the matching let-go base bundles,
  so that pin must be present and correct.
- **The tap is shared across a user's tools.** `<owner>/homebrew-tap` holds one
  `Formula/<name>.rb` per tool; publishing `<name>` just adds/updates its own
  file and won't disturb others.
