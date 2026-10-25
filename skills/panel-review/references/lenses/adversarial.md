## Lens: adversarial

Read the change as someone trying to break it, not as someone trying to approve it. Assume the
author's reasoning is plausible and still wrong somewhere.

Work the seams:

- the assumption the change depends on that nobody stated, and what happens when it does not hold
- inputs the author did not consider: empty, huge, malformed, duplicated, out of order, hostile
- the failure path nobody exercised, and the state it leaves behind
- what happens on a retry, on a restart, or with two of these running at once
- trust boundaries: data arriving from a caller, a file, the network or a subprocess without validation
- what the tests assert versus what the code actually promises. A passing test is not a proof

Do not repeat what a careful first read already surfaces. Your value is the finding the other
reviewers will not have.

**Attack the change hard; rate it the same as everyone else.** The stance decides what you look for,
never the severity you give it. An adversarial reading tends to inflate: the effort spent constructing
a trigger makes it feel likely. So before writing a severity, state the trigger to yourself in one
sentence. If it needs a precondition this codebase does not produce (a hostile process in a directory
only the user writes, a platform the project does not support, two operations the tool never runs
together), it is not `major`, and the body should say what would have to be true first. `critical` is
reserved for what is dangerous on an ordinary path, not "the worst thing I found".

**Title the mechanism you demonstrated, not the disaster you can construct from it.** "The two paths
open the file with different flags" is what you established; "the user's data is deleted" is a
consequence you inferred. Put the mechanism in the title, the consequence in the body, and mark which
part is observed and which follows from it.
