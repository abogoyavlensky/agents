# Two real backlog entries

Both come from the clojure-pulse-vscode repo. Match their register: concrete,
cites code by path, states how narrow the problem is, and either sketches the
fix with notes for the next person or explains the dead end.

## Example 1 — known fix, deferred on scope

    # Format Selection indents a mid-line top-level form from column 0
    
    **Status: open**
    
    ## Problem
    
    `formatRange` (the cljfmt engine, `src/fmt/cljfmtEngine.ts`) hands cljfmt a
    slice that starts at the top-level form's opener. cljfmt then indents the
    form's body as if it sat at column 0, so when the form does *not* start its
    line every inner column comes out short by the form's real column.
    
    Two ways to hit it:
    
    ```clojure
    ^:private (a
    b)        ; Format Selection puts `b` at column 1, cljfmt puts it at 11
    
    (x) (b
    c)        ; Format Selection puts `c` at column 1, cljfmt puts it at 5
    ```
    
    Glued reader prefixes (`#_`, `'`, `#?`, `@`, `#(`) are already correct: the
    backward walk in `formatRange` extends the slice over them.
    
    Narrow in practice — it needs Format Selection, or `editor.formatOnPaste`
    (which routes to the same provider), over a top-level form that does not begin
    its line. Format Document is unaffected, which is why the code carries this as
    a noted limitation rather than a bug.
    
    ## Fix
    
    Pad the slice with spaces up to the form's column before calling
    `reformatString`, then strip the same padding from the output. This leans on
    the property `formatRange` already relies on: cljfmt never reindents the first
    line of its input, so a slice that begins at the form's real column produces
    the columns the whole file would.
    
    Measured against whole-file cljfmt:
    
    | input | today | padded | whole-file cljfmt |
    | --- | --- | --- | --- |
    | `^:private (a\nb)` | `b` at 1 | `b` at 11 | `b` at 11 |
    | `(x) (b\nc)` | `c` at 1 | `c` at 5 | `c` at 5 |
    | `#_(a\nb)` | `b` at 3 | `b` at 3 | `b` at 3 |
    | `#(a\nb)` | `b` at 2 | `b` at 2 | `b` at 2 |
    
    Notes for whoever picks this up:
    
    - Keep the glued-prefix backward walk and pad only the columns before it, so
      the prefix reaches cljfmt verbatim. Padding alone matched all four cases
      above, but that is too little evidence to delete the walk.
    - Leave the "whitespace-only prefix" branch alone: a top-level form indented by
      mistake moves to column 0 there, matching whole-file cljfmt.
    - Cover reader-conditional splicing (`#?@(:clj [a\nb])`) in the tests; it was
      never actually exercised.
    
    About six lines in `formatRange` plus unit cases in
    `src/test/cljfmtEngine.test.ts` that compare `formatRange` output with
    `reformatString` over the whole text.
    
    ## Origin
    
    Raised by a Codex review during the indent-on-paste work (2026-09-03), which
    reported it as glued reader prefixes being dropped. That part was a false
    positive; the column offset behind it is real.

What makes it good: the repro is two lines; the cause is a named function in
a named file; "narrow in practice" tells triage how much it matters; the fix
names the invariant it relies on and comes with measurements, a do-not-touch
list, a test to add, and a size; the origin admits the review that raised it
was half wrong.

## Example 2 — deliberately left alone

    # Calling a `def`-prefixed function highlights its first argument as a name
    
    **Status: open**
    
    ## Problem
    
    `(defenders team)` — a call to an ordinary function whose name happens to start
    with `def` — is painted as a definition: `defenders` gets
    `keyword.control.clojure` and `team` gets `entity.global.clojure`, the scope the
    name in `(defn team ...)` would get.
    
    The cause is the `meta.definition.global` pattern in the `sexp` rule
    (`syntaxes/clojure.tmLanguage.json:303`), which opens on any paren-headed symbol
    matching `def[\w\d._:+=><!?*-]*` and scopes the first symbol inside as the
    defined name:
    
    ```
    (?<=\()(ns|declare|def[\w\d._:+=><!?*-]*|[\w._:+=><!?*-][\w\d._:+=><!?*-]*/def[\w\d._:+=><!?*-]*)\s+
    ```
    
    The related let-binding and argument cases — `(let [defenders 1] ...)`,
    `(pick defenders team)` — were fixed by requiring head position
    (`docs/plans/2026-09-04-0948-let-binding-def-highlight.md`). This one survives
    that fix because the call *is* in head position.
    
    ## Why it is left alone
    
    A TextMate grammar sees no more than the text. It cannot tell a call to a
    function named `defenders` from a use of a user-defined macro, and user-defined
    `def*` macros are everywhere: `defroutes`, `defstate`, `defentity`, `deftest`,
    `defcomponent`. Replacing the `def*` wildcard with a whitelist of known forms
    would mishighlight all of them to fix a rarer case — functions named `def*` are
    unusual, since the prefix conventionally means "this defines something".
    
    A real fix needs semantics, not text: semantic tokens from `clj-pulse`, which
    knows whether the head resolves to a macro or a function. That is the route to
    take if this is ever picked up.
    
    ## Origin
    
    Noticed while narrowing the `keyfn` patterns to head position (2026-09-04).

What makes it good: it links the sibling cases that *were* fixed and the plan
that fixed them, so nobody re-opens those; "Why it is left alone" explains
what a naive whitelist would break and names the only real fix (semantic
tokens from the server), so the next reader either has that tool or moves on.
