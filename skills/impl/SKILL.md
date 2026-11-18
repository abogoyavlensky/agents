---
name: impl
description: Implement a shape doc from docs/shapes/ in one run and exactly one PR — work through its items in reasonably sized checkpoints, each ending in the project's own check command, a commit and a background codex review whose findings land before the next commit; then a whole-diff review, end-to-end verification the agent does itself, the Result written back into the shape, lasting rules promoted to the project's roadmap, lessons kept in docs/KNOWLEDGE.md, leftovers sent to the backlog, and a PR. Use this whenever the user says "impl", "implement the shape", "build the shape", "ship this shape", "run the shape", or points at a file in docs/shapes/ and asks for it to be done — and offer it after the `shape` skill writes a doc. Not for step-by-step plans in docs/plans/ (that is executing-plans) or for /goal contracts in docs/goals/.
---

# Impl: Build a Shape in One Run, One PR

A shape (written by the `shape` skill) pins what will be true for the user and the rules the work must follow, and leaves the route to you. Your job is to choose that route well, prove each part works the way this project proves things, get a second opinion often, and leave the shape as an accurate record of what was built.

**Announce at start:** "I'm using the impl skill to build this shape."

**Two tracking surfaces.** The shape doc is the durable record: ticked items rewritten to say what was built, deviations, and the Result. The session task list (TaskCreate/TaskUpdate, if available) is how the user watches progress live — one task per checkpoint. If they disagree, the shape wins.

## 1. Load and check the shape

1. Read the shape. If none is named, take the most recent file in `docs/shapes/` whose status is `shaped`, and say which one.
2. Read what it leans on: the project's agent instructions (AGENTS.md, CLAUDE.md), its product rules or roadmap, its design source, and `docs/KNOWLEDGE.md` if it exists (lessons earlier runs paid for: test-harness gotchas, tool quirks). The shape is short *because* these exist; you must honour them as much as the shape.
3. **Check staleness.** Compare the shape's base commit with the current default branch (`git log --oneline <base>..origin/<default>`) and re-verify the facts each item states against the code. A fact that changed is fine to adapt to if the intent survives; note it.
4. **Check the Exit criteria.** Confirm the check command, e2e harness and test conventions still exist as written (CI workflow, task runner, instructions). If they are missing or wrong, rediscover them the same way the shape skill does — CI first, then agent instructions and README, then the task runner — and fix the shape.
5. Raise concerns with the user only if an item cannot be built as shaped, or two items contradict. Otherwise proceed.

**Resuming.** A long run can outlast a session or its context. If the shape's status already says `in progress on <branch> · k/N`, do not start over: check out that branch, read its `git log` against the default branch, look for review reports that finished but were never applied (`.tmp/codex-review-*.md` newer than the last commit that addressed them), apply those first, and continue from the first unticked item.

## 2. Branch and plan the checkpoints

1. Update the default branch and create a branch from it named after the shape (`shape/<topic>` or the project's own convention). Never work on the default branch, and do not let the new branch track it (`git switch -c <branch> origin/<default>` sets that up; `git branch --unset-upstream` undoes it), so a bare `git push` can never land on the default branch.
2. Set the shape's status to `in progress on \`<branch>\` · 0/<N>` (N = the number of items) and commit the shape as the branch's first commit, so the doc ships in the same PR as the work.
3. **Group the items into checkpoints.** Items are numbered; refer to them by number in checkpoint names, commit messages and progress notes ("Items 3 and 4"). A checkpoint is one item, or a few small related ones, that forms a coherent, reviewable change — roughly what a reviewer can read in one sitting. Put foundations first (schema, API, shared components that later items use). Create one session task per checkpoint. The grouping is yours; it does not go into the shape.

## 3. The checkpoint loop

For each checkpoint:

1. **Implement it, with tests.** Follow the conventions the shape names and the ones you see beside the code you touch — same kind of test, same place, same helpers. Test the behaviour the item promises, including its rule (the permission, the limit, the failure case), not just the happy path. If the project has an e2e harness, add a spec for each new user-facing flow as you build it.
2. **Run the check command until it passes.** Read its exit code directly — never through a pipe that hides it. A failing check is never committed. Some checks compare against git — a generated-code drift check (`git diff --exit-code` on generated paths) fails on any uncommitted regeneration — so stage the checkpoint (`git add -A`) before running the check, then commit what you staged.
3. **Commit** with a message saying what the checkpoint does for the user.
4. **Start `review-with-codex` on that commit in the background** (scope: that commit's sha) and move on to the next checkpoint without waiting. A purely mechanical checkpoint — data lists, strings and translations, renames, regenerated code — may skip its own review and be covered by the next checkpoint's (`--base` at the commit before it). Anything with logic, state, permissions or async behaviour gets its own.
5. **Before the next checkpoint's commit, fold in the review.** Read the finished report, verify each finding against the code (it is a second opinion, not ground truth), and apply the real ones as a small commit of their own (`Address review of <checkpoint>`). Keep it apart from the next checkpoint's work in progress: stash that work first (`git stash push -- <paths>`, then `git stash pop` after the fix commit), or stage only the fix's paths. The fix commit needs only the checks for the side it touches (say, the frontend's lint and unit tests): a partial tree can fail the full check for reasons that belong to the stashed work, and the next checkpoint's full check covers everything. When a fix adds a regression test, confirm the test fails without the fix (stash the fix, run that one test) when that is cheap — otherwise it may not test what the review found. If the fix and the new work touch the same lines, fold the fix into the checkpoint commit instead and say so in its message. Skip nits. Record each finding you reject, and each edge case you knowingly leave open, with its reason: in the fix commit's message, or under the item as a deviation. An edge case accepted only in your head is the one the final review finds again. If the review is still running when you are ready to commit, wait for it then — that is the only point where it blocks. If codex is unavailable, say so once and continue without it.
6. **Update the shape.** Tick the items the checkpoint completed and rewrite each to say what was actually built — the real behaviour, in the same register and about the same length as the shaped item (endpoints and file names belong in the Result and the commits, not in every item). Record any deviation under its item as `> Deviation: <what changed and why>`. Bump the status count (`· 3/7`). Commit the doc with the checkpoint or its review fix.

**Checks that already fail.** If a check fails in a way your change cannot explain, run it on the base (stash your work, or check out the base) before digging. A failure that reproduces on the base is not yours: fix it only if it is small and stands in the way of verifying the shape (a flaky or racy test, say), and record it as a deviation; otherwise stop and tell the user.

## Deviations: what you may decide, when to stop

- **Decide yourself** when the intent survives: an Open item, a fact that turned out slightly different, a better place for a shared piece, an extra small component. Record it under the item.
- **Stop and ask** when the change would break a Decision, cross a Non-goal, change a UX principle, change who can do something, or need a new dependency the shape did not mention.
- **Scope pressure:** a shape ships as exactly one PR. If the work grows past what one PR can carry, do not open a second one and do not silently drop an item. Stop, tell the user which items you would move into a new shape, and let them choose.

## 4. Finish: prove it works

Checkpoint-level green is not the same as the batch working. Before the PR:

1. Fold in the last checkpoint's review.
2. **Run one whole-diff review**: `review-with-codex` against the default branch (`--base <default>`), and wait for it. Codex versions differ in which scope flags accept a custom prompt — some reject one with `--base` — so run it without a prompt unless the installed version is known to take one. Per-commit reviews cannot see problems that cut across checkpoints — two items that both touch the same screen or the same list. Fix what is real.
3. **Run the full check command** once more on the final tree.
4. **Verify end to end yourself:**
   - **With an e2e harness**: run it, then actually look at the screenshots or artifacts it produces (open the images), and compare them with the project's design source where one exists. A passing suite with a broken layout is not done.
   - **Without one**: drive the app the way a user would — the `run` skill, a browser preview, the CLI — exercise each item, and record what you saw. Note in the Result that the project lacks a harness.
   - **What needs a human** (a real device, a third-party client, a live deploy, a real payment): do not pretend. List it as an owner check.

## 5. Record the result

In the shape:

1. Fill **Result**: what shipped (a few lines, not a file list), deviations gathered in one place, owner checks still to do, follow-ups with their backlog paths, and **one line on what the shape could have said better** (or "nothing") — this is how future shapes improve.
2. Set the status to `done: <PR URL>` once the PR exists, as a tiny follow-up commit pushed to the same branch. Never amend or force-push a branch that is already pushed. If the user asked you not to push or not to open a PR, set it to `done on \`<branch>\`, not pushed` instead and stop after this step.

Outside it:

- **Promote lasting rules.** A decision that future work must keep honouring ("Undo exists only on Add") belongs in the project's roadmap or product-rules doc, not only in an old shape. Add it there. If the shape completes items the roadmap lists, tick them.
- **Send leftovers to the backlog** with the `backlog` skill: real issues found along the way that are out of scope, and review findings you deliberately deferred. Each entry is its own commit, per that skill.
- **Keep what the run learned** in `docs/KNOWLEDGE.md`, creating it if needed: project facts that cost time to discover and will cost the next run the same — a test-harness gotcha, a check that needs files staged, a library quirk. One bullet each, under a short heading by area, saying what happens and what to do instead, and why when it is not obvious. Not product decisions (those go to the roadmap) and not what the code already says. Edit an existing bullet rather than adding a near-duplicate. List the new ones in the Result. Do not put them in AGENTS.md or CLAUDE.md.

## 6. Open the PR

Skip this step if the user asked you not to push or open a PR; report the branch instead.

Push the branch and open exactly one PR against the default branch, following the project's PR conventions (title style, template, attribution lines). The body is short: why (from the shape), what shipped (from Result), how it was verified (commands and e2e evidence), owner checks, and a link to the shape doc. Link the PR with whatever the host provides for that. Then report back: the PR URL, the checkpoints, the deviations, and the owner checks.

## When to stop and ask

- An item cannot be built as shaped, or two items contradict.
- A deviation would break a Decision, a Non-goal or a UX principle.
- The check command fails repeatedly for reasons outside the change.
- The work outgrows one PR.

Don't force through a blocker, and don't guess at a product decision. Stop, say what you found, and offer the options.

## Key principles

- **The shape is the contract; the route is yours.** Honour its Decisions and Non-goals as strictly as the project's own invariants.
- **Small checkpoints, frequent second opinions.** Review runs in the background; findings always land before the next commit.
- **Prove it the project's way, then look with your own eyes.** Check command, tests beside the code, e2e you actually inspect.
- **Leave an accurate record.** The ticked shape says what was built, not what was planned.
- **One shape, one PR.**
