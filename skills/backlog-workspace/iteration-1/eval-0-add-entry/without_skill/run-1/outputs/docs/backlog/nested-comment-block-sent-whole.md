# Evaluate Top Form sends a nested `(comment …)` whole

**Status: open**

## Problem

A rich comment block inside another one does not get the body-form treatment
the outer one gets. With the cursor inside the inner comment's body, Evaluate
Top Form sends the whole inner `(comment …)` — which evaluates to `nil` —
instead of the body form under the cursor:

```clojure
(comment
  (comment
    (+ 1 |2)))   ; sends `(comment (+ 1 2))` → nil, not `(+ 1 2)` → 3
```

The one-level descent is deliberate today, not an oversight. `topFormAtCursor`
(`src/repl/forms.ts:320`) descends into a top-level comment once and hands the
cursor to `resolveCommentBody` (`src/repl/forms.ts:366`), which returns the
body form containing the cursor without looking at what that form is. The
doc comment says so ("One level only: a nested comment is sent whole"), the
plan chose it (`docs/plans/2026-09-05-2253-evaluate-top-form.md`, "Comment
descent"), and a test pins it:

```ts
assert.strictEqual(top("(comment (comment |x))"), "(comment x)");
```

(`src/test/forms.test.ts:426`).

The report named Evaluate Current Form, but that command has no comment
handling at all — `formAtCursor` resolves the innermost form wherever the
cursor is, nested comment or not. Only Evaluate Top Form treats a comment
block specially, so it is the one to change.

## Fix

Make the descent recursive: after `resolveCommentBody` picks a body form, if
that form is itself a bare `(comment …)` (`commentHeadEnd`,
`src/repl/forms.ts:341`, already decides that) and the cursor is between its
brackets, resolve again inside it with the same rules. Every rule then holds
at every depth, in particular:

- On the inner `comment` head, or before its first body form, the inner
  comment form is the result — the same fallback the outer level has.
- A prefixed or qualified inner comment (`#_(comment …)`,
  `(clojure.core/comment …)`) is still sent whole; `commentHeadEnd` handles
  that unchanged.

Then flip the pinned test to expect `x`, add a two-deep case for the head
fallback, and drop "one level only" from the `topFormAtCursor` doc comment
and from the README's Evaluate Top Form entry (`README.md:487`, `:693`),
which say the forms "directly under `comment`" count as top level.

## Origin

Noticed while testing rich comment blocks after the Evaluate Top Form command
shipped (`docs/plans/2026-09-05-2253-evaluate-top-form.md`, 2026-09-09).
Left alone in that PR.
