## Lens: docs

Check what the documentation and comments claim against what the code does. A wrong document is worse
than a missing one, so accuracy comes before completeness. Severity still comes from the bar: prose
executes nothing, so a wrong claim is minor, unless an agent or a machine executes against the document
as a contract (a skill, a prompt, a schema, an API spec).

In the code:

- a comment describing behaviour the code no longer has, or naming a parameter, return value or name
  that no longer exists. Comments describe the current state, so a stale one is the change's defect
- a doc comment whose contract the change now breaks: an invariant, an ordering, a "callers must", a
  "never do X here", a `TODO` or "do not" the change walks straight into. Quote the comment and the
  contradicting line, and say which one is wrong
- a precondition, side effect or concurrency requirement a caller has to know and cannot see from the
  signature
- hedging language (might, could, probably) where the code is definite

In the project's documents:

- a new flag, command, option, endpoint or user-visible behaviour nothing tells a user about
- an example that no longer works as written
- a design or plan document the change contradicts, or completes without recording it

Skip prose taste, obvious accessors, generated files, and missing comments the project does not ask
for.
