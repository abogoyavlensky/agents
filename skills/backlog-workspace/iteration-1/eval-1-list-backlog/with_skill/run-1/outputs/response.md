Four open entries in `docs/backlog/`, none planned or done:

1. **Palette commands show in workspaces with no Clojure in them** — `docs/backlog/palette-activation-gate.md`
   `contributes.commands` is static, so all twenty "Clojure Pulse" palette entries appear in Python/TypeScript workspaces too. Proposed fix: set a `clojurePulse.active` context key at activation and gate each `menus.commandPalette` entry on it. Size: one `setContext` call, twenty manifest `when` clauses, and a richer visibility rule in `src/test/manifest.test.ts`.

2. **Format Selection indents a mid-line top-level form from column 0** — `docs/backlog/format-selection-column-offset.md`
   `formatRange` hands cljfmt a slice starting at the form's opener, so `^:private (a\nb)` or `(x) (b\nc)` gets its body indented as if the form sat at column 0. Hits Format Selection and `editor.formatOnPaste`; Format Document is unaffected. Fix: pad the slice with spaces to the form's real column, strip the padding from the output. Size: about six lines in `src/fmt/cljfmtEngine.ts` plus unit cases (the entry includes measurements against whole-file cljfmt for four inputs).

3. **Calling a `def`-prefixed function highlights its first argument as a name** — `docs/backlog/def-prefixed-call-highlighted-as-definition.md`
   `(defenders team)` paints `team` as a defined name because the grammar's `def*` wildcard can't tell a function from a user macro. Deliberately left alone: a whitelist would break `defroutes`, `deftest`, `defstate` and friends; the real fix is semantic tokens from clj-pulse.

4. **The `jar` scheme is hardcoded twice instead of read from the server** — `docs/backlog/jar-scheme-hardcoded-not-negotiated.md`
   clj-pulse advertises `textDocumentContentProvider.schemes: ["jar"]` but `src/extension.ts` and `src/client.ts` hardcode it. Deliberately left alone: nothing breaks until a second scheme exists, and honouring the capability would need a fallback for older servers anyway, so it'd be more code, not less.

I checked both fix-carrying entries against the current tree (a few commits have touched `package.json` and `cljfmtEngine.ts` since they were written): neither has been fixed incidentally. There's no `clojurePulse.active` anywhere, and `formatRange` still carries the comment saying space-separated prefixes like `^:private (…)` are not recovered. `src/test/cljfmtEngine.test.ts` exists but has no `formatRange` cases yet, which matches the entry.

**I'd pick up the palette activation gate first.** Three reasons:

- **Widest reach.** It's the only one of the four that every user sees — anyone who installs the extension and then opens a non-Clojure project gets twenty irrelevant palette entries. The format-selection bug is real but you have to use Format Selection or format-on-paste on a form that doesn't start its own line.
- **Lowest design risk.** The pattern already exists in the codebase (`clojurePulse.hasInlineResults` in `src/repl/inlineResults.ts`), the activation events already describe exactly the workspaces that should see the commands, and the entry spells out the two traps (combine with the existing `editorLangId == clojure` guard on `showClojureDocs`; rewrite the manifest test's visibility rule rather than dropping the explicit twenty-command list). It's mechanical.
- **Context is fresh.** It was deferred from the palette trim that shipped in #26 a few commits ago, so the manifest and test are still shaped the way the entry describes.

The format-selection fix is the close second, and honestly small enough (six lines plus tests, with the expected columns already measured) that I'd do it in the same sitting right after. I wouldn't touch the other two: both have a "Why it is left alone" section that holds up, and picking them up would mean either a semantic-tokens project or a dynamic-registration protocol change, neither of which is justified by the current symptom.

Say the word and I'll triage the palette gate into a plan.
