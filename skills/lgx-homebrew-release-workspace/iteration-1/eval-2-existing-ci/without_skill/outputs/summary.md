# Homebrew release pipeline for gizmo

Pushing a version tag `vX.Y.Z` now builds cross-platform binaries, publishes a
GitHub Release, and updates the Homebrew tap so users can
`brew install acme/tap/gizmo`.

## Files added / changed (in `project/`)

- `.github/workflows/release.yml` — new release workflow, tag-triggered (`v*`).
- `.github/scripts/render-formula.sh` — renders the Homebrew formula from the
  version + per-archive SHA-256 checksums.
- `README.md` — added an Installation section (`brew install acme/tap/gizmo`).
- `.github/workflows/checks.yml` — **untouched** (only read). The release
  workflow *reuses* it via `workflow_call` so releases run the existing checks
  first instead of duplicating them.

## What the workflow does (jobs in `release.yml`)

1. **check** — `uses: ./.github/workflows/checks.yml` (fmt check + tests). The
   release proceeds only if checks are green.
2. **build** — matrix over `darwin/amd64`, `darwin/arm64`, `linux/amd64`,
   `linux/arm64`, all on `ubuntu-latest`:
   - `lgx install` fetches deps.
   - Downloads the matching let-go base `let-go_<ver>_<os>_<arch>.tar.gz` from
     `github.com/nooga/let-go` (version read from `.mise.toml`, so it stays in
     sync), extracts the `lg` binary.
   - Stamps the tag version into `main.lg`, then cross-compiles with
     `lgx build -bundle-base <lg>`.
   - Packages `bin/gizmo` as `gizmo_<version>_<os>_<arch>.tar.gz`.
3. **release** — downloads all archives, writes `checksums.txt` (sha256),
   creates the GitHub Release (archives + checksums) with `gh release create`,
   then renders `Formula/gizmo.rb` and commits/pushes it to the tap repo.

## Manual steps you must do once

1. **Create the tap repo** `github.com/acme/homebrew-tap` (public, can be empty;
   the workflow creates `Formula/gizmo.rb` on the first release). The name
   `homebrew-tap` is what makes `brew install acme/tap/gizmo` resolve.
2. **Add a push token for the tap.** In the `acme/gizmo` repo settings, add an
   Actions secret **`HOMEBREW_TAP_TOKEN`** — a PAT (or GitHub App token) with
   `contents: write` on `acme/homebrew-tap`. The default `GITHUB_TOKEN` cannot
   push to another repo, so this is required.
3. **Cut a release:** commit your changes, then
   `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. **Install:** `brew install acme/tap/gizmo` (or
   `brew tap acme/tap && brew install gizmo`).

## Assumptions / notes

- Cross-compilation is done entirely on Linux runners via `-bundle-base` (the
  let-go base binary IS the target-platform runtime), so no macOS or arm64
  runners are needed.
- let-go release asset naming assumed as documented:
  `let-go_<version>_<os>_<arch>.tar.gz`, tag `v<version>` (with a fallback to a
  non-`v` tag). Adjust in `release.yml` if nooga/let-go uses a different scheme.
- The formula's Homebrew fields (`desc`, `homepage`, owner/repo) default to the
  `acme/gizmo` values inside `render-formula.sh`; override via env if needed.
- Per constraints, no git tag was created and no release command was run.

## Verification done in the sandbox

- YAML of `release.yml` parses; `checks.yml` confirmed unchanged.
- `render-formula.sh` output verified (valid Homebrew `on_macos`/`on_linux` +
  `on_arm`/`on_intel` blocks; Ruby `#{bin}` interpolation preserved).
- Version-stamp `sed`, the `.mise.toml` version grep (-> `1.11.1`), and the
  checksum `awk` lookup all verified against the real files.
- The `lgx build -bundle-base` flag/semantics confirmed via `lgx build --help`.
- End-to-end binary build could NOT be run here: this sandbox's `lg` 1.11.1
  fails to AOT-compile the project (`Can't resolve core/run`), and the same
  error hits `lgx build`, `lgx test`, and `lgx run` identically — reproducible
  in a minimal isolated example. This is a pre-existing sandbox/runtime issue,
  not a change I made, and does not affect the correctness of the pipeline
  files. It should build normally in real CI where `lgx test` already passes.
