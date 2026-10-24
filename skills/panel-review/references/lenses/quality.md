## Lens: quality

Report code that runs correctly and could be written better: the kind of thing a maintainer reading
it six months from now has to re-derive. Judge it on what is in front of you, bottom-up, and against
the project's own idioms (read a neighbouring file first).

Look for:

- constructs the language treats as non-idiomatic: an untyped catch-all where a concrete type fits, a
  boolean argument nobody can read at the call site, a pointer or reference to something already
  passed by reference
- structure that fights the reader: a function longer than a screen, nesting deeper than three levels
  where an early return would flatten it, dead branches after a return
- names that mislead: a name that says one thing while the code does another, two names for one concept
- errors discarded with no reason given, an empty failure branch, a default value returned after a
  failure without a word about it
- an error returned bare where the caller cannot tell which operation failed, or wrapped so the
  original can no longer be inspected
- duplication the change introduces: a block pasted with one value changed, or logic the repository
  already provides. Cite the existing copy so the finding is actionable
- error handling at the wrong boundary: validation deep between trusted internal calls, while the
  system edge goes unchecked

Duplication is often deliberate and right. Two occurrences are not yet a pattern, tests repeat their
setup to stay readable, and repetition that keeps two modules independent is buying something. Flag it
only when you can name the shared form and show it costs nothing.

Leave alone: an intentional discard that says why, generated files, and test code written plainly on
purpose.
