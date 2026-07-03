# Homebrew release pipeline for gizmo — summary

Added a tag-driven Homebrew (+ mise) release pipeline to the `gizmo` CLI
(`github.com/acme/gizmo`) using the `lgx-homebrew-release` skill. The existing
CI workflow was preserved, not overwritten.

## Resolved project values

| key        | value                                             |
|------------|---------------------------------------------------|
| name       | gizmo (from `lgx.edn :targets {:bin {:out "bin/gizmo"}}`) |
| owner/repo | acme/gizmo                                         |
| desc       | Do the thing across all your repos                |
| tap        | acme/homebrew-tap  ->  `brew install acme/tap/gizmo` |
| homepage   | https://github.com/acme/gizmo                     |
| lg version | 1.11.1 (from `.mise.toml`)                         |
| version    | 0.1.0                                              |

## Files created (via render.py)

- `scripts/generate-formula.sh` — renders the Homebrew formula from a release
  `checksums.txt` (executable, class `Gizmo`).
- `scripts/tag.lg` — tags `v<VERSION>` from `resources/VERSION`.
- `.github/workflows/release.yml` — build (4 targets: linux/darwin x
  amd64/arm64) + GitHub Release + Homebrew-tap-update jobs. Its `checks` job
  reuses the existing `checks.yml` via `uses: ./.github/workflows/checks.yml`.
- `resources/VERSION` — `0.1.0`, verified 5 bytes with NO trailing newline.

## Existing CI preserved

- `.github/workflows/checks.yml` was LEFT UNTOUCHED (render.py skipped it because
  it already exists — still has the `# gizmo project CI - keep me` comment). It
  already declares `workflow_call:`, so `release.yml` calls it as-is. No changes
  needed.
- I intentionally did NOT add the optional lint step to `checks.yml` that the
  skill mentions, to honor "do not overwrite / the user is happy with it." If
  wanted later, add `- run: lgx lint` between the fmt and test steps.

## Files edited (single-source the version)

- `main.lg` — added `[io :as io]` to the `ns` requires (kept sorted), added
  `(defn- version [] (io/slurp (io/resource "VERSION")))`, and changed the app
  spec to `:version (version)` (was hardcoded `"0.1.0"`).
- `lgx.edn` — added `:resource-paths ["resources"]`, `:lg-version "1.11.1"`,
  and a `release` task (`scripts/tag.lg` + `git push --tags`).
- `README.md` — added an Installation section (Homebrew / mise / manual).

## Verification (in this sandbox)

- `lgx fmt check` — PASS (all source files formatted correctly).
- `lgx lint` — PASS (0 errors, 0 warnings).
- YAML parse of both workflows — PASS.
- Formula generator smoke test with a synthetic `checksums.txt` — PASS
  (emits a valid `class Gizmo < Formula` with correct urls/version/desc).
- `resources/VERSION` — 5 bytes, no trailing newline (verified with `od -c`).
- `lgx build` / `lgx test` — FAIL with `Can't resolve core/run in this context`.
  This is a PRE-EXISTING sandbox/environment limitation, NOT a regression:
  reverting `main.lg` to the original hardcoded-version baseline reproduces the
  exact same error. `ruby`/`brew test` were not run (no ruby toolchain here).

## Manual steps the user must do (cannot be automated)

1. Create the tap repo (once, if it doesn't exist): a public repo named exactly
   `homebrew-tap` under the `acme` account/org. It can start empty; the workflow
   creates `Formula/` on first run.
2. Add the `HOMEBREW_TAP_TOKEN` secret in the gizmo repo (Settings -> Secrets and
   variables -> Actions -> New repository secret): a PAT with write (`contents`)
   access to `acme/homebrew-tap`. `GITHUB_TOKEN` is provided automatically and
   only covers the release itself.
3. Cut the first release:
       printf '0.1.0' > resources/VERSION   # or bump to the next version
       git add -A && git commit -m "Release 0.1.0"
       lgx release                          # tags v0.1.0, pushes -> triggers release.yml
   NOTE: no tag was created or pushed here (per task constraints). After the run
   completes, `brew install acme/tap/gizmo` (and
   `mise use -g github:acme/gizmo@latest`) will work.
