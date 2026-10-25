# Review profile template

Use this to draft `docs/review-profile.md` for a repository. Every finder and verifier reads it, so it
is where "what matters here" stops living in someone's head. Fill it from the README, AGENTS.md /
CLAUDE.md, the code layout and recent history, then show the draft to the user before writing it.
Keep it under ~60 lines; finders read it on every review.

```markdown
# Review profile: <project>

## What this is
<Two or three sentences: what the project is, who runs it, where it runs. A CLI one person runs on
their own machine and a multi-tenant server need very different reviews.>

## What a real failure looks like
<Concrete, ranked by cost. What would actually hurt here: silent data loss, a wrong answer believed, a
broken public API, a migration that cannot be rolled back, a skill an agent executes wrongly…>

- **<failure>**: <why it costs, where it would come from>

## Blast radius
<Who is affected when something breaks, and how bad that is. What is and is not a security boundary.
Which inputs are untrusted.>

## Where the project's rules live
<Paths to the authoritative docs: AGENTS.md, CLAUDE.md, docs/…, linters' configs. Say which one wins
when they disagree.>

## Always worth reporting
<Project-specific rules whose violation is a finding even when it looks minor, e.g. "every change to a
CLI flag must update README and docs/", "no new dependencies without a note in docs/decisions/".>

## Not worth reporting
<Things reviewers keep raising that this project has settled, e.g. "test helpers are duplicated on
purpose", "we do not wrap errors in the cli/ package".>
```
