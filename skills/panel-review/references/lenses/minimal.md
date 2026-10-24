## Lens: minimal

Check that the change added only what the task required. Code is easier to generalize once a real
second use arrives than to un-generalize when the abstraction was wrong, and every extra line is paid
for by whoever reads it next.

Look for:

- premature abstraction: a helper extracted for one caller, generic parameters with no second user, a
  type hierarchy for a single concrete case, a factory serving one implementation
- speculative generality: options or flags nothing exercises, branches for hypothetical future inputs,
  an extension point nothing extends
- defensive checks against impossible states: guards for conditions the surrounding code or the types
  already prevent
- backwards-compatibility scaffolding for code that never shipped, or an old function kept after every
  caller moved to its replacement
- a wrapper that only forwards, or a layer that passes its argument through untouched
- comments that restate the next line instead of capturing a non-obvious reason

Name the smaller shape in the fix: which lines go, what the call site becomes. "Three similar lines"
beats a premature abstraction, so do not suggest extracting one either.
