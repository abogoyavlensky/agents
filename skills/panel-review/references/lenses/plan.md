## Lens: plan

Judge whether the document is complete, consistent and ready to build from. Flag only what would cause
a real problem during implementation: an implementer, often an agent with no other context, follows
the text literally.

Look for:

- **completeness**: `TODO`, `TBD`, placeholders, a section that promises content it does not have, a
  requirement in the design that no task implements
- **consistency**: two sections or tasks that contradict each other: a name, path, signature, data
  shape or behaviour stated one way here and another way there
- **clarity**: a requirement ambiguous enough that two careful implementers would build materially
  different things
- **executability**: a step whose command, path or expected output is missing or wrong; tasks in an
  order where one depends on something a later task creates; a verification step that cannot fail
- **scope**: several independent subsystems crammed into one plan, so no task boundary is a safe place
  to stop
- **YAGNI**: features, options or abstractions the stated goal never asked for

Not findings: minor wording, stylistic preferences, sections that are less detailed than others, and
details the plan deliberately leaves to the implementer, who will settle them correctly by reading the
code.
