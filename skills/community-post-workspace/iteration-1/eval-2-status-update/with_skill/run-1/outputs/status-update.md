**What's in the works** (late August)

A quick look at what's cooking in lgx right now:

- 📦 **Transitive deps from `deps.edn` / `project.clj`** - merged on master. Add a Clojure lib and lgx now reads the manifest it ships for other tools: git coords resolve directly, well-known Maven coordinates get pinned through a curated registry of verified repo+tag mappings, and anything lgx can't pin produces one warning naming the exact coord to add or exclude. That's the top item on the roadmap, done.

- 🐹 **Go deps** - in progress on a branch. Declare a Go package as a coord in `lgx.edn`, e.g. `modernc.org/sqlite {:go/version "v1.57.0"}`, and lgx builds a cached custom `lg` runtime that links it, with optional generated bindings via `:go/interop`. The proof case is SQLite: a pure-Go driver in a single static binary, no cgo, no C toolchain. `run`, `repl`, `test`, and `build` pick the runtime up transparently.

- 🛠 **Cross-compilation** - same branch. `lgx build --target linux/arm64` builds for one platform ad-hoc, `lgx build --all` walks a `:platforms` matrix declared in `lgx.edn`, all from one machine. Works with Go deps too, as long as they're pure Go.

- 🧹 **`lgx clean`** - same branch. Reclaims the `$LGX_HOME` caches (`--runtimes`, `--gitlibs`, `--templates`, or `--all`), with `--dry-run` to see sizes before deleting. Never runs automatically.

**Released vs not:** none of this is in v0.1.2, the current release. Transitive deps discovery will ship in the next lgx release. The Go deps, cross-compilation, and clean work also depends on interop support landing in a let-go release first, so it's a bit further out.

If any of this would unblock something you're building, say so here, it helps with prioritizing. 🙏
