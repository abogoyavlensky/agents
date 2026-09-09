# Evaluate Top Form sends a nested `(comment …)` whole instead of the body form under the cursor

**Status: open**

## Problem

Inside a top-level rich comment block, Evaluate Top Form sends the body form
under the cursor. When that body form is itself a `(comment …)`, the descent
stops there: with the cursor inside the inner comment's body, the whole inner
`(comment …)` is sent and evaluates to `nil`, instead of the form under the
cursor like the outer block gives.

```clojure
(comment
  (comment
    (+ 1 |2)))   ; sends (comment (+ 1 2)) => nil; the outer block alone would send (+ 1 2)
```

The cause is `topFormAtCursor` in `src/repl/forms.ts:320`. It reads the top
form, checks once whether it is a bare `(comment …)` list (`commentHeadEnd`),
and if so returns the body form the cursor is in (`resolveCommentBody`). That
body form is returned as-is; it is never checked for being a comment list in
its turn. The docstring states the rule: "One level only: a nested comment is
sent whole", and `src/test/forms.test.ts:424` pins it under "descent is one
level only and never recursive":

```ts
assert.strictEqual(top("(comment (comment |x))"), "(comment x)");
```

This was a design decision in
`docs/plans/2026-09-05-2253-evaluate-top-form.md` ("Comment descent is one
level only ... Nested comments are not descended"), not an oversight in the
walker, but the plan gives no reason for stopping at one level.

The report attributed this to Evaluate Current Form; that is not where it
lives. Evaluate Current Form (`formAtCursor`, `src/repl/forms.ts:300`)
resolves the innermost form at the cursor and does not know about `comment`
at all: `(comment (comment (+ 1 |2)))` gives `2` from it, the same as it would
with no comment around it. Only Evaluate Top Form has the descent, so only it
has the one-level limit.

Narrow in practice: it needs a `comment` block nested directly inside another
one, which happens when someone wraps part of a rich comment to park it, or
pastes one rich comment into another. Everything else already works: a single
level of `comment` descends correctly, deeper non-comment nesting
(`(comment (a (b |c)))` → `(a (b c))`) is the intended one-level rule for
ordinary forms, and prefixed or qualified comments (`#_(comment …)`,
`(clojure.core/comment …)`) are deliberately sent whole.

## Proposed fix

Loop instead of checking once: after `resolveCommentBody` yields a body form,
run the same `commentHeadEnd` check on it, and if it is a bare `(comment …)`
list with the cursor strictly inside its brackets, resolve its body the same
way; repeat until the resolved form is not a comment list. Each iteration
uses the invariant the current code already relies on — the cursor is
between the list's `(` and `)`, so `resolveCommentBody` either finds the body
form containing the cursor or the previous one. The cursor-on-the-head and
before-the-first-body-form fallbacks stay per level: a cursor on the inner
`comment` symbol yields the whole inner comment, matching what the top-level
rule does today.

Notes for whoever picks this up:

- `topFormAtCursor` is the only caller path (`evalTopForm` in
  `src/extension.ts:1591`); `formAtCursor`, Select Current Form and the form
  highlight are untouched.
- Flip the pinned case in `src/test/forms.test.ts:424` from `(comment x)` to
  `x`, keep `(comment (a (b |c)))` → `(a (b c))` as-is, and add the
  cursor-on-inner-head case (`(comment (comm|ent x))` → `(comment x)`) and an
  unbalanced inner body (`(comment (comment (a |` → null).
- Update the docstring on `topFormAtCursor` and the README wording at
  `README.md:487` ("the forms directly under `comment` count as top level")
  to say the descent repeats through nested comments. Leave the "Comment
  descent" line in the plan alone; it is the record of the original decision.

About eight lines in `topFormAtCursor` (a loop around the existing
`commentHeadEnd` / `resolveCommentBody` pair) plus the test changes above.

## Origin

Reported while testing rich comment evaluation after Evaluate Top Form
shipped (`docs/plans/2026-09-05-2253-evaluate-top-form.md`, PR #29),
2026-09-09. The report named Evaluate Current Form; the behavior is Evaluate
Top Form's, and Evaluate Current Form is unaffected.
