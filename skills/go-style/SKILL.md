---
name: go-style
description: How to write Go in Andrey's preferred style - boring, idiomatic, data-oriented Go with explicit data flow, explicit wiring and lifecycle, contained mutation, small consumer-side interfaces, and no framework magic - plus version-aware modern Go idioms and a verification loop. Use this whenever you write, edit, refactor, fix, or add tests to Go code - any task touching .go files or go.mod, Go HTTP services and handlers, CLIs, workers, storage code, or Go project layout - even when the user doesn't mention style. Load it before writing the first line of Go in a session.
---

# Go style

The goal is idiomatic Go that an experienced Go developer understands at once,
with a few ideas borrowed from Clojure systems: explicit data flow, values
treated as immutable by convention, functions over data, and an explicit
system graph (config → resources → services → handlers → server) wired in
plain code, Integrant-style, with no container.

The question to ask for every design choice:

> What is the simplest Go code that makes the data flow, dependencies, and
> lifecycle obvious?

Agents write much of this code, and humans have to review it. Repetitive but
explicit code is cheap for you to produce and easy for them to read; clever
abstractions are the opposite. Optimize for the reviewer.

`references/style-guide.md` has the full rationale and worked examples (a
composition root, lifecycle, storage, HTTP handler, worker, tests). Read the
relevant section when you are building one of those pieces from scratch or
the rule below isn't enough to decide.

## Workflow

1. **Orient before writing.** Read `go.mod` (the `go` version, and which
   libraries are already in use: pgx or `database/sql`, testify, router,
   logger), the task runner (`Makefile`, `Taskfile.yml`, `justfile`), lint
   config (`.golangci.yml`), and a couple of files next to where you'll work.
2. **Match the project on surface choices** — libraries, layout, naming,
   test style. Existing conventions beat this guide there. Apply the
   structural rules below to the code you write; don't refactor unrelated
   code into this style unless asked.
3. **Write the code** following the rules below and the Go version you found.
4. **Verify** with the loop at the end, then re-read your diff against the
   "avoid by default" list.

## Rules

**1. Prefer boring Go.** Standard library first: `net/http` and `ServeMux`,
`log/slog`, `context`, `errors`, `encoding/json`. Add a dependency only when
it clearly earns its place, and mention it to the user when you do — new
modules are a decision, not a detail.

**2. Make data flow explicit.** Functions take the data and dependencies
they need and return results. An operation should read top to bottom:
decode → normalize → validate → construct → persist → result. A reader
should never have to reconstruct hidden behavior.

**3. Treat structs as values; contain mutation.** Pass and return small
domain structs by value. Use pointers when identity matters, the type owns a
resource or lock, or copying is wrong. Local mutation is fine
(`u.Email = ...; return u`), but never quietly modify caller-owned data —
slices and maps especially. If a function must mutate its argument, its name
and doc comment say so.

**4. Pure core, infrastructure at the edges.** Domain logic works on plain
values and is trivially testable. Databases, HTTP, env vars, clocks, and
randomness live at the boundaries. Side effects should be visible from call
sites and package names (`user.Validate` vs `store.Insert` vs `mailer.Send`).

**5. Wire the system explicitly.** One composition root (e.g.
`internal/app`) builds the graph in ordinary code: config → db/logger →
stores → services → handlers → server. No DI containers, service locators,
registries, or `init()` side effects. The root owns every long-lived
resource and closes them in reverse order with a shutdown timeout. `main`
stays boring: signal context, load config, build app, run — inside a
`run() error` so defers execute.

**6. No package-level mutable state.** No global db, config, or logger. State
that must be shared (cache, mutex, queue) lives in a type whose purpose is
owning that state, constructed in the composition root.

**7. Services are structs; transformations are functions.** A service holds
its dependencies (`user.Service{store, logger}`) and its methods orchestrate.
The steps they call — normalize, validate, compute — are free functions over
values. Don't invent receiver types as pseudo-namespaces; packages are
namespaces. A type holding a mutex gets pointer receivers.

**8. Interfaces are small, consumer-side, and on demand.** Write concrete
code first. Define an interface where it's consumed, with only the methods
that consumer calls, when there's a real second implementation or a test
fake. No generic repositories, no interface-per-struct.

**9. Named types where mixing would be a bug.** `user.ID`, `site.ID` instead
of bare strings when IDs from different domains could be swapped. Not a
wrapper for every primitive.

**10. Errors are values with context.**
- Wrap at meaningful boundaries: `fmt.Errorf("find user %s: %w", id, err)`.
  Lowercase, no "failed to"/"error:" prefix — the chain reads as a sentence.
- Sentinel or typed errors only when a caller branches on them
  (`ErrNotFound`, `*ValidationError`); map driver errors (`sql.ErrNoRows`,
  `pgx.ErrNoRows`) to domain errors in the storage package.
- **Log or return, never both.** The boundary that handles an error (the HTTP
  error mapper, the worker loop, `main`) logs it once.
- No panics for ordinary failures; only for broken invariants.

**11. `context.Context` is for cancellation, deadlines, and request-scoped
metadata** — first parameter, never stored in structs, never a dependency
bag (`ctx.Value("db")`).

**12. Packages by domain, not by layer.** `internal/user`, `internal/site`,
`internal/httpapi`, `internal/postgres`, `internal/app`, `cmd/<name>` — not
`models/ services/ repositories/ controllers/`. No `utils`, `common`,
`helpers`, or `pkg/`. Names don't stutter: `user.Service`, not
`user.UserService` (the package's primary type, like `user.User`, is the
accepted exception).

**13. SQL stays visible.** Write SQL directly (pgx, `database/sql`, or
generated sqlc code — whatever the project uses; pgx for new Postgres
projects). No ORM, no query-builder framework, no generic repository. SQL
lives in the storage package as named constants next to the function that
runs it.

**14. HTTP handlers are thin adapters.** Decode → call service → map
result/error → encode. Business decisions don't accumulate in handlers. Use
`ServeMux` method/path patterns (`"GET /users/{id}"`, `r.PathValue`) on Go
1.22+ instead of adding a router.

**15. Concurrency at the edges, with an owner.** Synchronous code by default.
Goroutines for serving, independent I/O, workers, bounded pipelines. Every
long-lived goroutine answers: who starts it, who cancels it, who waits for
it, where do its errors go. Channels are a concurrency primitive, not an
abstraction; always handle a closed channel (`v, ok := <-ch`).

**16. Config is data.** A typed `Config` struct loaded and validated once at
startup, then passed down (or used to build dependencies). Nothing reads env
vars after startup.

**17. Constructors are plain.** `NewX(deps..., cfg XConfig) *X` with explicit
parameters or a config struct. No functional options in application code —
they hide what a component needs; leave them to libraries with many optional
knobs.

**18. Generics and helpers only when they pay.** A small `Map`-style helper
or a generic container is fine if used in several places. No type-level
frameworks, no FP helper libraries (lo, etc.) — a `for` loop with `append` is
the idiomatic `map`/`filter`.

**19. Comments explain why, not what.** Doc comments on exported
identifiers, starting with the name. Don't narrate obvious code.

## Tests

- Test behavior through public functions and service methods, not internal
  structure. Table-driven tests when cases share a shape; plain tests when a
  table would obscure them.
- testify (`require` for preconditions that must stop the test, `assert` for
  the checks) is fine; follow the project if it uses stdlib only.
- Hand-written fakes over mock generators: a `fakeStore` struct with a map
  satisfies the small consumer-side interface in a few lines and doesn't
  couple tests to call order. No gomock/mockery.
- Storage code is tested against a real database using the project's
  existing setup (docker compose, testcontainers). Don't mock SQL.
- Tests construct what they need; no shared package state. Use `t.Helper()`,
  `t.Cleanup`, and `t.Parallel()` where tests are independent.

## Write for the module's Go version

Use the newest idioms the `go` line in `go.mod` allows — and nothing newer.
Don't bump the `go` version unless asked.

| Version | Use |
|---|---|
| 1.21 | `min`/`max`/`clear` builtins; `slices`, `maps`, `cmp` packages instead of `sort.Slice` and hand-rolled loops; `log/slog`; `context.WithoutCancel`, `context.AfterFunc` |
| 1.22 | Per-iteration loop vars (drop `tt := tt`); `for i := range n`; `ServeMux` patterns `"POST /users/{id}"` + `r.PathValue("id")`; `math/rand/v2` |
| 1.23 | Range-over-func iterators (`iter.Seq`), `slices.Collect`, `slices.Sorted(maps.Keys(m))` |
| 1.24 | `tool` directives in go.mod (`go get -tool`, `go tool staticcheck`); `t.Context()`; `b.Loop()`; `omitzero` JSON tag; `strings.SplitSeq`/`Lines`; `os.Root` |
| 1.25 | `wg.Go(func() {...})`; `testing/synctest` for time-dependent tests; `http.CrossOriginProtection` |
| 1.26 | `new(expr)` for pointer-to-value (`new(30)`); `errors.AsType[*ValidationError](err)`; `go fix` modernizers |

## Verify

Prefer the project's task runner targets when they exist. Otherwise:

```bash
gofmt -l .                  # must print nothing; fix with gofmt -w <files>
go vet ./...
go test -race ./...         # drop -race only if cgo is unavailable
go mod tidy                 # only if you changed dependencies
```

Also run the project's linter if configured (`golangci-lint run`,
`staticcheck ./...`). With a Go 1.26+ toolchain, run `go fix -diff` on the
packages you touched and apply the modernizations that land in your own
changes — don't rewrite untouched code in an unrelated diff.

Report failures you couldn't fix instead of hiding them.

## Avoid by default

Each needs a concrete, demonstrated reason before it appears:

- DI containers, service locators, registries, `init()` with side effects
- ORMs, generic repositories, query-builder frameworks
- package-level mutable state; globals in tests
- `utils`/`common`/`helpers` packages, `pkg/`, layer-named packages
- reflection- or annotation-driven behavior; frameworks that own lifecycle
- speculative interfaces, interface-per-struct, mock generators
- functional options in app code; FP helper libraries
- channels as a general abstraction; fire-and-forget goroutines
- logging an error and also returning it
- `context.Value` for dependencies
- mutating arguments the caller still owns
- value receivers on types that contain a mutex
- new dependencies the user didn't agree to
- microservices, distributed queues, or code generation layers before the
  workload needs them

When two designs are close, pick the one with fewer hidden rules: explicit
wiring, data, errors, and lifecycle. Explicit doesn't mean verbose — a small
helper that removes real repetition while keeping the flow visible is
welcome.
