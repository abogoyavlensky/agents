# lgx status update — what's in the works

Hey everyone! Quick update on where lgx is at and what's cooking.

## Recently shipped (v0.1.2)

- **Maven deps from Clojure libs** — lgx now resolves `mvn` dependencies declared by Clojure libraries, both directly and transitively, so more existing libs work out of the box.
- **Built-in lib template** — `lgx new -t lib` scaffolds a library project.
- **More compatibility examples** — malli, aero, and integrant now have worked examples under `examples/`, joining tools.cli and bond. If you're wondering whether your favorite Clojure lib runs under let-go, that's the place to look.

## In the works (not released yet)

The big one: **Go dependencies as first-class lgx deps**. The idea is that you declare a Go package right in `lgx.edn` next to your git and local deps:

```clojure
{:deps {database/sql       {:go/interop "sql"}
        modernc.org/sqlite {:go/version "v1.38.0"}}}
```

On first `lgx install`, lgx generates a Go module, builds a custom `lg` runtime with those packages linked in (and interop bindings generated where you ask for them), caches it, and transparently uses it for `run`/`repl`/`test`/`build` — including single-binary `lgx build` output. After the first build everything is cached, so day-to-day commands feel exactly like before.

Riding along on the same branch:

- **Cross-compilation** — `lgx build --target <platform>` and `--all` to produce binaries for other platforms from one machine, with `:platforms` and templated `:out` paths in config. Early measurements: a cold cross-build is ~10–15s, a cache hit is free.
- **`lgx clean`** — an explicit cleanup command for the runtime, gitlibs, and template caches.
- **A SQL layer for let-go** — a driver-agnostic `sql` package with a next.jdbc-shaped API plus thin sqlite (pure-Go modernc.org/sqlite) and postgres (pgx) drivers, built on the new `:go/*` deps. Working end to end as the proving ground for the whole pipeline.

Fair warning: the Go-deps work depends on interop changes landing upstream in let-go, so no promises on timing — it ships when both halves are ready.

## Also on the roadmap

Transitive deps from `deps.edn`/`project.clj` libs, `lgx install --all`, `lgx deps` (dependency tree), and `lgx update`.

---

If any of this sounds interesting — especially the Go-deps design or the SQL API shape — feedback now is the cheapest it'll ever be. Nothing is tagged yet, so every signature is still up for discussion. And if you've tried a Clojure lib under let-go that isn't in the examples yet, tell us how it went!
