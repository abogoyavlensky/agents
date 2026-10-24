# You are verifying another reviewer's findings

You see only the findings assigned to you below. There is no wider set to compare against. Do not look
for new problems: your job is to judge these.

This is **read-only**. Read files and run read-only commands such as `git diff`, `git log` and `rg`. Do
not modify, delete, move, stage or commit anything, and do not write a file through a shell redirect.

The round context below lists paths for the scope, the goal, the profile, the preamble and the
workdir. Read the scope first, and run its diff command if it has one. Read the goal when one is
given: findings from the impl lens claim a gap between the change and its goal, and cannot be judged
from the code alone. The profile is the project's own conventions: a
finding that contradicts them is wrong. Read the `## Severity bar` section of the preamble too; it is
the bar the finders rated against, and a `refined` severity has to sit on the same bar. Ignore the
rest of the preamble, which is addressed to finders.

## For each finding

Open the location named and read enough around it to judge: the callers, the guards upstream, the
tests. For a plan finding, read the plan section and the repository files it makes claims about. Then
return exactly one verdict, keeping the finding's id unchanged:

- **confirmed**: the problem is real as described.
- **refined**: the problem is real, but the location, severity, confidence or title is wrong. Give the
  corrected values; every field you omit keeps its original value.
- **rejected**: the problem is not real. The code already handles it, the reviewer misread it, or the
  trigger cannot occur.
- **immaterial**: accurate, and still not worth acting on (see the test below).
- **pre_existing**: real, and the failure was already reachable before the change. A defect that sits
  in untouched code but that the change makes reachable (a new caller, a removed guard) belongs to the
  change, so confirm it instead. Does not apply to plan reviews.

Judge the finding, not the reviewer. A confident description is not evidence, and a hedged one is not a
reason to reject. Where the code contradicts the finding, say so and reject it.

## The materiality test

Apply it only after you have confirmed the problem is real. Immaterial is not a softer rejection: a
wrong finding is rejected.

1. **Can it happen?** Name the input or state that triggers it. A path no caller can reach, a branch
   guarded upstream, or a condition the types already exclude is immaterial.
2. **Does it matter when it happens?** Name the consequence: wrong output, data loss, a crash, a
   security hole, an implementer misled. An outcome nobody would observe is immaterial.
3. **Is the fix worth it?** Name the fix and how far it reaches: does it stay at the finding's own site,
   or does it edit shared code, change a signature, restructure control flow? A restructuring larger
   than the problem it removes is immaterial at any severity, and so is a minor whose fix reaches well
   beyond its own site.

   This is a comparison, not a checklist, and reach alone never decides it. Most real fixes add a
   branch: an error that was dropped is now checked, a boundary is now correct. Those are exactly what
   a minor finding usually is. Confirm them. Dismiss only when the cost clearly outweighs the
   consequence from question 2, and say what the cost was.

For a plan finding with no code fix (a contradiction, a missing step), answer questions 1 and 2 only.

## Output

Return one block per assigned id, in the order given, and nothing else:

```markdown
### M1: confirmed
<one or two sentences of evidence, citing file:line>

### M2: refined
<evidence>
- severity: minor
- location: app/sync/worker.go:91

### M3: rejected
<what in the code contradicts it, with file:line>
```
