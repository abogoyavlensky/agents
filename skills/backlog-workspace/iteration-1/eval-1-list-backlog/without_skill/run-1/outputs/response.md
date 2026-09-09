## What's in the backlog

All four entries in `docs/backlog/` are open:

1. **`format-selection-column-offset.md`** — Format Selection indents a mid-line top-level form from column 0. `formatRange` (`src/fmt/cljfmtEngine.ts`) hands cljfmt a slice starting at the form's opener, so `^:private (a\nb)` or `(x) (b\nc)` gets its body indented as if the form sat at column 0. Only hits Format Selection / `formatOnPaste` over a form that doesn't begin its line; Format Document is unaffected.

2. **`palette-activation-gate.md`** — Palette commands show in workspaces with no Clojure in them. `contributes.commands` is static, so all twenty "Clojure Pulse" entries appear in a Python/TypeScript workspace. Fix: set a `clojurePulse.active` context key on activation and gate every `menus.commandPalette` entry on it. Deferred from the palette trim (#26).

3. **`def-prefixed-call-highlighted-as-definition.md`** — Calling a function whose name starts with `def` (`(defenders team)`) is highlighted as a definition, because the grammar's `meta.definition.global` pattern opens on any `def*` head symbol. Deliberately left alone: a TextMate grammar can't distinguish this from user `def*` macros (`defroutes`, `deftest`, ...), so a real fix needs semantic tokens from `clj-pulse`.

4. **`jar-scheme-hardcoded-not-negotiated.md`** — The `jar` scheme is hardcoded in `src/extension.ts` and `src/client.ts` instead of read from the server's advertised `textDocumentContentProvider.schemes` capability. Nothing is broken; the two agree by coincidence. Left alone until a second scheme exists, which none is planned.

## Which I'd pick up first, and why

**`format-selection-column-offset.md`.**

- It's the only one that is a genuine, user-visible wrong result: formatting produces incorrect columns. The palette one is cosmetic noise, and the other two are explicitly parked ("why it is left alone") with no good fix available at the extension level.
- It's the most fully specified. The entry names the exact mechanism, the fix (pad the slice to the form's column, strip the padding from the output), the invariant it relies on (cljfmt never reindents the first line), a measured table against whole-file cljfmt, and the three traps to avoid (keep the glued-prefix walk, leave the whitespace-only branch, add a `#?@` test). It's estimated at about six lines in `formatRange` plus unit cases in `src/test/cljfmtEngine.test.ts`, and the code still matches that description (`formatRange` at line 208, `reformatString` call at line 258).
- Small blast radius: the change is local to one function and testable purely against `reformatString` on the whole text, so there's no VS Code integration to exercise.

Close second: **`palette-activation-gate.md`**. It's also small and well-scoped (the `setContext` pattern already exists in `src/repl/inlineResults.ts:311`), and it's a natural follow-up to the trim that just shipped in #26. The main cost is touching every palette-visible command in `package.json` and rewriting the visibility rule in `src/test/manifest.test.ts`, which is more churn for a cosmetic win. I'd do it right after the formatting fix.

I would not pick up the `def`-prefix highlight (needs semantic tokens from `clj-pulse`, a different project) or the `jar` scheme negotiation (no second scheme exists, and the entry argues convincingly that reading the capability would add code rather than remove it) until their preconditions change.
