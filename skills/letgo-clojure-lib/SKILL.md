---
name: letgo-clojure-lib
description: >-
  Add a worked example of a Clojure library running under let-go (driven by lgx)
  to examples/clojure-libs/, AND check whether let-go itself needs upstream fixes
  to support that library — routing any needed runtime/compiler/type changes to
  the fastplan skill. Use this whenever the user wants to add, port, try, or
  smoke-test a Clojure library under let-go or lgx; asks "does <lib> work under
  let-go / lg?"; wants a `with-<lib>` example; or wants to "make <lib> compatible
  with let-go" — even if they only say "add an example for <lib>". It runs the
  library under the locally-built patched `lg` to surface compatibility gaps,
  classifies each gap, hands any let-go work to fastplan, then scaffolds and
  verifies the example end-to-end.
---

# Adding a Clojure library under let-go

Two intertwined jobs: (1) produce a `examples/clojure-libs/with-<lib>/` example
that runs a real Clojure library under let-go via lgx, and (2) find out whether
let-go needs upstream changes to load/run it — and if so, hand that work to
**fastplan** rather than hacking it inline.

The order matters: you can't write a working example until the library actually
loads and runs under `lg`, and that often requires let-go changes first. So the
real spine of this skill is **finding compatibility gaps by running the library
under the patched `lg`**, then routing and verifying.

## Why running it is the whole game

Static reading only gets you so far. let-go's `require` **silently swallows a
load failure** — it prints `error: failed to load ...` to stderr but does *not*
throw, and `find-ns` still returns the namespace. So "it loaded" is never proof.
The reliable signal is: run the library, read the **first** error, fix/route it,
re-run, and repeat until it loads *and* a small functional smoke actually works.
Grepping let-go tells you if a symbol exists; only running tells you if the
library behaves.

## Environment (orient first)

- **lgx repo**: the one containing `examples/clojure-libs/`. Examples live at
  `examples/clojure-libs/with-<lib>/` as `{lgx.edn, main.lg}`.
- **let-go checkout + built `lg`**: the patched runtime you test against, at
  `/Users/andrew/Projects/let-go` (referred to below as `<letgo>`); the binary is
  `/Users/andrew/Projects/let-go/lg` (referred to below as `<lg>`). Confirm the
  binary exists and is freshly built (`cd /Users/andrew/Projects/let-go && make
  build`). **Rebuild after every let-go change** — a stale `lg` will mislead you.
  (If that checkout isn't present, ask the user where their let-go is.)
- **`LG_READ_CLJ=1`** must be set for the spawn so `.clj` files resolve and
  `:clj` reader-conditional branches match. lgx sets it automatically; when you
  invoke `lg` directly, set it yourself.
- **No transitive deps**: lgx does not resolve a plain Clojure lib's transitive
  dependencies, so the example's `lgx.edn` must list **every** dep directly
  (e.g. integrant → also list `dependency`), and a direct `lg` probe needs
  every dep on `-source-paths`, joined with `:`.

Consult the repo's own notes when a surprise hits — don't preload them:
`docs/knowledge-base/let-go-gotchas.md`, `let-go-resolver.md`,
`let-go-stdlib-quick-ref.md`, `lgx-dev-workflow.md`, and prior write-ups in
`docs/issues/` (e.g. `integrant-dependency-compat.md`, `clojure-lib-compat.md`)
— they record gaps already solved and the house style for reporting new ones.

## Workflow

### 1. Pin the library and get its source

Get the coord (`owner/repo` + a `:git/tag` or `:git/sha`) and the entry
namespace(s). Fetch the source once: reuse the lgx cache if present
(`~/.lgx/gitlibs/<host>/<owner>/<repo>/<ref>/src`), else shallow-clone
(`git clone --branch <tag> --depth 1 <url> <tmp>`). Read the main `.cljc`/`.clj`
file to see what it `:require`s and which host/JVM surfaces it touches
(`clojure.lang.*`, `java.*`, reader conditionals, `defrecord`/`deftype`,
`defmulti`, `defprotocol`).

### 2. Probe: load it under `lg` and read the first error

```
LG_READ_CLJ=1 <lg> -source-paths "<dep1-src>:<dep2-src>" \
  -e "(require '<entry.ns>) (println :loaded)"
```

Then **functionally smoke it** — call the library's headline fns on real input
(build its core value, run its main operation) — because a namespace can register
after a swallowed load error. Fix/route the first error, rebuild `lg` if it was a
let-go change, re-run. Repeat until it loads clean AND the smoke works.

For each `Can't resolve <sym>`: check whether let-go has it —
`grep -rn '"<sym>"' <letgo>/pkg/rt/lang.go <letgo>/pkg/rt/core/*.lg` and
`grep -rn 'defn <sym>\b' <letgo>/pkg/rt/core/*.lg`. Absent → a gap.

### 3. Classify each gap — and decide *where the fix lives*

Two questions per gap: *what kind* is it, and *which layer* should own it —
`.lg` (`pkg/rt/core/*.lg`) or Go (`pkg/rt/*.go`, `pkg/vm/`).

**Default to `.lg`; use Go only when the gap genuinely needs it.** let-go's
self-hosted IR pipeline lowers `.lg` to native Go via the AOT path
(`gogen_ir` / `lower_go.lg`), so a pure-Clojure implementation is *not* slower
than hand-written Go — and it stays code the language can see, test, and optimize
through its own pipeline. The maintainers explicitly want as much of let-go as
*can* be written in let-go to live in `.lg`
([nooga/let-go #519](https://github.com/nooga/let-go/issues/519) is the canonical
statement of the boundary — read it when a fix's home is unclear). Every gap that
lands in Go for no reason is code the language can't reach.

| Gap kind | Signal | Home (why) |
|---|---|---|
| Already present | grep finds it | — use it |
| Pure stdlib fn | `Can't resolve <fn>`; a plain Clojure `defn` over existing let-go fns would do it (e.g. `class`, `uri?`, `monitor-enter`) | **`.lg`** — lowers to native Go; no reason to touch Go |
| Stdlib fn needing one primitive | the body is Clojure but one step needs a vm capability it can't reach (a type check, a namespace/registry lookup) | **`.lg` fn + one tiny exported Go primitive** — the reflection-kernel pattern (#519 item 3): export `public-vars`/`indexed?`-style building blocks from Go, write the fn in `.lg` |
| Load-only / throw-on-call stub | a JVM-only surface is *referenced* but not runnable (classpath scan, reflection, a formatter built at load) | **`.lg` compat-stubs module** (`pkg/rt/core/compat_stubs.lg`, #519 item 4). Go only for the `DefNSBare` kernel that *creates* a bare host namespace (`clojure.lang.RT` etc.) — the stub fn bodies are `.lg`. (The older all-Go `ArrayList`/`RT` stubs are the pattern this supersedes.) |
| Reader / compiler gap | a valid Clojure form won't compile (empty `catch`, field scoping) | **Go compiler** (runs before `.lg` loads) — but IR-side desugaring can be `.lg` where the pipeline allows (#476 catch-desugaring precedent) |
| New value type | a real mutable/collection type is needed (a working queue/deque/map) | **Go** *for now* — a `.lg` `deftype` can't yet emit the full collection interface set (#519 item 2); note it as a future `.lg`-deftype candidate so it's not forgotten |
| Interop dispatch / host-class & host-method / static-registration tables | `.method` on a native value; `(instance? Class x)`; `Class/staticFn` resolution | **Go** — init-time machinery, correctly Go (#519 non-goals). If the static-method surface keeps *growing*, propose **one bulk-registration helper**, not another wall of one-off `ns.Def`s |
| Degraded-by-design | loads but can't fully work | note it; keep the example off that path |

Everything still routes to **fastplan** (step 4) — the point of this table is to
tell fastplan *which layer* to plan for.

Prefer fixes that **fail loudly** over ones that return plausible-but-wrong
values. A stub that throws when called beats one that silently lies.

### 4. Route let-go work to fastplan

If **any** gap needs a let-go change, don't edit let-go here. Write a tight gap
report — for each: a minimal repro, the exact error, its **home** (`.lg` vs Go,
per step 3) with a one-line reason, and a proposed fix (file + approach) — then
invoke the **fastplan** skill with it, noting the plan targets the let-go repo
(`<letgo>`). Making the home explicit is what steers fastplan to the right layer;
default it to `.lg` and justify any Go landing. fastplan produces the plan;
executing-plans implements it; then rebuild `lg` and return to step 2 to confirm
the library now loads and runs. A concrete gap report makes fastplan far more
effective, so invest in it. (Reminder for the implementer: editing
`pkg/rt/core/*.lg` requires regenerating the bundle —
`go run -tags bootstrap ./cmd/lgbgen` — before `go test`/`make build`; a fix that
lands mostly in `.lg` needs this, a pure-Go fix does not.)

**New-file header.** Any **new Go source file** the fix adds (`pkg/rt/*.go`,
`pkg/vm/*.go`) opens with the project's standard MIT header, attributed
generically to **`let-go contributors`** — not an individual's name — so
attribution stays uniform across the tree (this is what recent let-go files
already do). Use the current year. New `pkg/rt/core/*.lg` files carry no such
header. Fold this into the gap report so the implementer applies it:

```
/*
 * Copyright (c) 2026 let-go contributors
 * SPDX-License-Identifier: MIT
 */
```

If **no** let-go change is needed (the library already works, or after fixes
land), go straight to the example.

### 5. Scaffold the example

Mirror the existing examples — read one first as a template:
- `with-dependency` / `with-medley` — small, pure-data libs (build a value, query it).
- `with-integrant` — a stateful lifecycle (init/halt, refs, multimethods).

Create `examples/clojure-libs/with-<lib>/`:

- **`lgx.edn`** — `:main "main.lg"`, a `:targets {:bin {:out "bin/with-lib"}}`,
  and `:deps` listing the library **and all its transitive deps** directly, each
  pinned by `:git/tag` (or `:git/sha`).
- **`main.lg`** — `(ns main (:require [<lib.ns> :as x]))`, a tiny `demo`/`println`
  helper, then exercise the library's **headline** features on real input with
  labeled output. Keep it readable and comment *what each feature demonstrates*,
  matching the neighbors' voice. If you use a `-main`, guard the entry with
  `(when-not *compiling-aot* (-main))` — top-level side effects otherwise fire
  during `lg -b`/AOT.

### 6. Verify end-to-end (both ways)

- **Direct (fast loop):**
  `LG_READ_CLJ=1 <lg> -source-paths "<all-dep-srcs>" examples/clojure-libs/with-<lib>/main.lg`
- **Through lgx (faithful):** build lgx if needed (`make build`), then
  `cd examples/clojure-libs/with-<lib> && LGX_LG=<lg> <lgx-bin> run main.lg`
  (lgx auto-installs deps and sets `LG_READ_CLJ`). Output must match the direct
  run.

Both must show the headline features actually working — not just ":loaded".

### 7. Docs (if let-go changed)

Per the repo's same-PR rule, when let-go gained a fn/behavior, add it to
`docs/knowledge-base/let-go-stdlib-quick-ref.md` and write/refresh a
`docs/issues/*.md` note (repro + fix + status), in the style of the existing
ones. Keep each doc's `Verify against:` footer accurate.

## Common gotchas (learned the hard way)

- **Silent require failure** — always functionally smoke; ":loaded" lies.
- **Rebuild `lg`** after any Go change; **regenerate the bundle**
  (`go run -tags bootstrap ./cmd/lgbgen`) after any `pkg/rt/core/*.lg` change.
- **`defrecord`/`deftype` field scope, `case` constants, empty `catch` body,
  a real `PersistentQueue`, `find-var`/`get-method`** — all were real let-go
  gaps found this exact way. Expect the next lib to surface its own; treat a new
  `Can't resolve`/compile error as a sub-gap to route, not a wall.
- **Report gaps so an upstream maintainer would accept them** — each as a
  standalone, general let-go improvement with a minimal repro, not "integrant
  needs it." Mention the library as motivation.
- **Land it in `.lg`, Go only when it must** — the maintainers are actively
  moving the language into itself (#519). A pile of one-off `ns.Def`s or a Go
  loud-stub that a plain `.lg` `defn` could express reads as landing in the wrong
  layer, even if it works. Pick the home in step 3, state it in the report.
