# Homebrew release pipeline for `gizmo`

Set up a tag-driven Homebrew + mise release pipeline in the project, following
the `lgx-homebrew-release` skill.

## Resolved project values

| key        | value                                                 |
|------------|-------------------------------------------------------|
| name       | `gizmo`                                               |
| owner/repo | `acme/gizmo`                                          |
| formula    | `Gizmo` (CamelCase of name)                           |
| desc       | Do the thing across all your repos                    |
| tap        | `acme/homebrew-tap` -> `brew install acme/tap/gizmo`  |
| homepage   | https://github.com/acme/gizmo                         |
| lg version | `1.11.1` (from `.mise.toml`)                          |
| version    | `0.1.0` (from `main.lg`)                              |

## Files created (via `assets/render.py`)

- `scripts/generate-formula.sh` — renders the Homebrew formula from a release
  `checksums.txt` (executable, `class Gizmo < Formula`).
- `scripts/tag.lg` — tags `v<VERSION>` from `resources/VERSION`.
- `.github/workflows/checks.yml` — fmt + lint + tests, reusable via
  `workflow_call`.
- `.github/workflows/release.yml` — build (4 targets: linux/darwin x
  amd64/arm64) -> GitHub Release -> update tap formula.
- `resources/VERSION` — `0.1.0`, no trailing newline (verified: 5 bytes).

## Files edited

- `main.lg` — added `[io :as io]` to requires; version now read from the
  resource: `(defn- version [] (io/slurp (io/resource "VERSION")))` and
  `:version (version)` in the app spec (single source of truth).
- `lgx.edn` — added `:resource-paths ["resources"]`, `:lg-version "1.11.1"`,
  and a `release` task (`scripts/tag.lg` + `git push --tags`).
- `.github/workflows/checks.yml` — added a `Run lint` step (`lgx lint`) between
  fmt and tests, since the project has an `lgx lint` task.
- `README.md` — added an Installation section (Homebrew / mise / manual).

## Verification performed

- `lgx fmt check` — PASS (edits are well-formatted).
- YAML parse of both workflows — PASS (valid).
- `scripts/generate-formula.sh 0.1.0 <synthetic checksums>` — PASS (emits a
  valid `Gizmo` formula with the four platform URLs + sha256 blocks).
- `resources/VERSION` byte check — 5 bytes, no trailing newline.

### Pre-existing issue (NOT caused by this setup)

`lgx build` and `lgx test` fail in this sandbox with
`Can't resolve core/run in this context` at `main.lg` `(def app ...)`. This was
confirmed to reproduce with the fully original, unmodified project (original
`main.lg` + original `lgx.edn`), so it is a fixture/environment quirk in this
sandbox's let-go toolchain, independent of the release pipeline. `ruby`/`brew
style` were unavailable for formula linting (ruby not installed), so the formula
was validated by generating it and inspecting the output instead.

## Manual steps the user must do (outside the repo)

1. Create the tap repo — a public repo named exactly `homebrew-tap` under
   `acme` (i.e. `github.com/acme/homebrew-tap`). May start empty; CI creates
   `Formula/` on first release. Skip if it already exists.
2. Add the `HOMEBREW_TAP_TOKEN` secret — in the `acme/gizmo` repo:
   Settings -> Secrets and variables -> Actions -> New repository secret.
   Value: a PAT with write (`contents`) access to `acme/homebrew-tap` (a
   fine-grained PAT scoped to that repo is ideal). `GITHUB_TOKEN` is automatic;
   only the tap token is manual.
3. Cut the first release (do NOT run during setup):
   ```bash
   printf '0.2.0' > resources/VERSION      # bump, no trailing newline
   git add resources/VERSION && git commit -m "Release 0.2.0"
   lgx release                             # tags v0.2.0 + pushes -> triggers CI
   ```
   Then users install with `brew install acme/tap/gizmo` or
   `mise use -g github:acme/gizmo@latest`.

Per task constraints, no git tag was created/pushed and neither `lgx release`
nor `scripts/tag.lg` was run.
