# Article Review Checklist

Use this for the editorial and readability pass (Pass 3). Walk every item, note what is missing or weak, apply the clear wins, and surface the bigger moves as a short prioritized list with line references.

## Motivation and stakes (the "why")

- Does the opening make the problem concrete? A reader should know what hurts and why within the first few sentences.
- For how-to, tooling, or security posts, name the payoff early. Do not bury the actual point (the key command, the real risk) two-thirds of the way in.
- Example: a sandboxing post should say up front what an unsandboxed agent can actually do (delete files, read secrets, hit the network), not leave it implicit.

## Orientation

- For longer posts, is there a one-line TL;DR near the top? Match the author's existing TL;DR style.
- Do the title and intro let a reader self-select in or out?

## Concreteness

- Replace vague claims with specifics: numbers, durations, sizes, versions, command names, file paths.
- If one option gets hard numbers (for example "10-15 GB"), give the competing option the same treatment so the comparison is fair.
- Name the tool, flag, or file instead of "a tool" or "the config".

## Structure and ordering

- Does each step depend only on things already introduced? Watch for instructions that run inside an environment set up in a later step (for example, a command meant to run "inside the VM" shown before the "open a shell" step).
- Rationale before mechanics: decisions and why, then the commands.
- Flag duplicated explanations across sections. Preview-then-walkthrough is fine; re-teaching the same thing twice is not.

## Redundancy

- If one point appears three or more times (a recurring caveat or limitation), consolidate it to the single place where it lands best and reference it once.

## Honesty and completeness

- State limitations plainly. A short "what this protects and what it doesn't" beats overclaiming.
- Cover the obvious follow-up questions a reader will hit: auth and credentials, cleanup and teardown, recovering from a bad state.
- Do not oversell. Soften absolute claims the rest of the post contradicts (for example, "fully isolated" when egress is still unsolved).

## Links and references

- Link every named tool or project on first mention.
- Trim dangling "foo/bar/..." lists, or turn the entries into real links. No filler entries.

## Landing

- Does the post end on a wrap-up rather than a stray bullet? One honest sentence on where this nets out for the author.

## Apply vs propose

- Apply directly: clear readability wins, missing links, trimming filler, fixing ordering, de-duplicating.
- Propose first: new sections, removing content, reordering whole sections, or anything that changes scope or intent. Give a short prioritized list with line references and apply on the author's go-ahead.
