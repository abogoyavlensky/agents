# gizmo release pipeline — setup summary

Automated releases for `gizmo` (github.com/acme/gizmo) are now wired up: pushing a
`vX.Y.Z` tag builds cross-platform binaries, publishes a GitHub Release with the
archives + `checksums.txt`, and updates the Homebrew tap so users can
`brew install acme/tap/gizmo`.

The setup mirrors the author's existing lgx release pattern (as used in `wtr`),
adapted to `gizmo` / `acme`.

## Files added

- `.github/workflows/release.yml` — triggered on `v*` tags. Three jobs:
  - `checks` — reuses `checks.yml` (fmt + tests) as a release gate.
  - `release` — reads the pinned let-go version from `.mise.toml`, downloads the
    matching let-go base for each of `linux_amd64`, `linux_arm64`, `darwin_amd64`,
    `darwin_arm64` from `github.com/nooga/let-go` releases (verifying its
    `checksums.txt`), bundles the binary with `lgx build -bundle-base <lg>`, packs
    `gizmo_<version>_<os>_<arch>.tar.gz` into `dist/`, smoke-tests the native
    binary, writes `dist/checksums.txt`, and publishes the GitHub Release with
    `gh release create`.
  - `homebrew` — regenerates the formula from the release `checksums.txt` and
    commits `Formula/gizmo.rb` to `github.com/acme/homebrew-tap`.
- `.github/workflows/checks.yml` — CI on push/PR to `master`, also callable via
  `workflow_call`. Runs `lgx fmt check` and `lgx test`.
- `scripts/generate-formula.sh` — renders `Formula/gizmo.rb` (class `Gizmo`,
  macOS/Linux x intel/arm URLs + sha256) from a release `checksums.txt`.
- `scripts/tag.lg` — `lgx release` helper: tags `v<resources/VERSION>`.
- `resources/VERSION` — single source of truth for the version (`0.1.0`,
  no trailing newline).
- `.gitignore` — ignores build artifacts (`bin/`, `dist/`, `build/`, `bases/`, ...).

## Files modified

- `main.lg` — version now read from `resources/VERSION`
  (`(io/slurp (io/resource "VERSION"))`) instead of a hard-coded string, so the
  binary's `--version`, the git tag, and the Homebrew formula version all stay in
  sync from one place.
- `lgx.edn` — added `:resource-paths ["resources"]` (so `VERSION` is bundled) and
  a `release` task (`scripts/tag.lg` + `git push --tags`).
- `README.md` — added an Installation section (Homebrew, mise, manual download).

## Build-blocking bug fixed (required for releases to work)

The project as delivered did NOT build or test: `main.lg` and
`test/gizmo/core_test.lg` aliased `gizmo.core` as `core`, which collides with
let-go's built-in `core` namespace, so `core/run` never resolves
(`Can't resolve core/run in this context`). This is deterministic in let-go
1.11.1, so `lgx build` / `lgx test` — and therefore the release workflow — would
fail. I renamed the alias to `cmds` in both files. After the fix, verified locally:
`lgx fmt check`, `lgx test` (1/1 pass), `lgx build`, `bin/gizmo --version`
(-> `gizmo 0.1.0`), and `bin/gizmo run` (-> `gizmo ran`) all succeed.

## Verification done

- Full build path verified locally (fmt, test, build, `--version`, smoke test).
- `scripts/generate-formula.sh` produces a valid `Formula/gizmo.rb` from a mock
  `checksums.txt`; Ruby/brew environment confirmed healthy.
- Did NOT create/push any tag or run any release command (per constraints).
- `lgx -bundle-base` cross-compile could not be exercised end-to-end here (needs
  the downloaded per-target let-go bases / network), but the workflow mirrors the
  proven `wtr` pipeline exactly.

## Manual steps you must do

1. Create the tap repo `github.com/acme/homebrew-tap` (public, empty is fine;
   the workflow creates `Formula/gizmo.rb` on first release).
2. Add a repo secret `HOMEBREW_TAP_TOKEN` on `github.com/acme/gizmo`: a
   fine-grained/classic PAT with contents: write on the `acme/homebrew-tap`
   repo. The workflow uses it to push the updated formula. (The release itself
   uses the built-in `GITHUB_TOKEN`; no extra setup needed for that.)
3. Confirm the license. The formula declares `license "MIT"` (matching the
   author's other tools). If gizmo is not MIT, change the `license` line in
   `scripts/generate-formula.sh` and add a matching `LICENSE` file.
4. Cut a release (whenever ready): edit `resources/VERSION` to the new version,
   commit, then either run `lgx release` (tags `v<VERSION>` and pushes) or manually
   `git tag vX.Y.Z && git push origin vX.Y.Z`. Keep `resources/VERSION` and the tag
   in agreement — the Homebrew `test do` asserts the binary's `--version` matches
   the tag.
5. After the first successful release, `brew install acme/tap/gizmo` works on macOS
   and Linux (amd64/arm64).

## Notes

- The let-go runtime version is pinned in `.mise.toml` (`lg = "1.11.1"`); the
  release workflow reads it and pulls the matching bases. Bump it there to move to
  a newer let-go.
- CI checks run `lgx fmt check` + `lgx test` (matching the reference `wtr` setup).
  `lgx lint` (clj-kondo) is available locally via `lgx lint`/`lgx check` but is not
  run in CI.
