---
name: tech-article-writing
description: Review, edit, or draft technical articles, blog posts, tutorials, README-style walkthroughs, and documentation in Andrey Bogoyavlensky's practical technical writing voice. Use when asked to review or improve an article end to end (grammar, fluency, factual correctness, readability) and apply the fixes; improve grammar, fluency, structure, or style of technical prose; align a draft with "my style"; or write/rewrite articles for bogoyavlensky.com while preserving the author's voice.
---

# Tech Article Writing

## Core Approach

Make technical writing clear, practical, and human without turning it into corporate or generic AI prose.

When run against an article (a `.md` file, or "review/improve this post"), do a full end-to-end review: grammar, fluency, factual correctness, and overall readability, and apply the improvements. Default to actually improving the draft, not just listing suggestions.

Use this skill together with general clarity rules from `writing-clearly` when available. This skill controls voice, article shape, and the review workflow; `writing-clearly` controls grammar, concision, active voice, and avoidance of puffy wording.

## Review Workflow

Work in passes. Read the whole draft first. If you are in a repo with other articles, sample 2-4 nearby ones to lock onto the author's voice. Note the article's goal and audience before touching anything.

**Pass 1 - Copyedit (apply directly).**
- Fix grammar, spelling, punctuation, and awkward phrasing.
- Improve fluency and flow: omit needless words, prefer active voice and positive form, keep related words together (lean on `writing-clearly`).
- Fix internal consistency: names, paths, commands, and option flags should match across the whole article.
- Strip generic AI phrasing (`seamless`, `robust`, `leverage`, `delve`, `groundbreaking`, empty "-ing" clauses).

**Pass 2 - Factual correctness.**
- Sanity-check technical claims, commands, flags, and any version- or behavior-specific statements.
- If the article links to repos, docs, or other sources, open them (WebFetch) and verify that the claims, commands, and snippets actually match what the source says.
- Fix clear errors. For anything you cannot verify, flag it for the author instead of silently rewriting it into something that might also be wrong.

**Pass 3 - Editorial and readability review.**
- Walk the article against `references/review-checklist.md`: motivation/stakes, orientation, concreteness, structure and ordering, redundancy, honesty, links, and landing.
- Apply the clear wins directly. For bigger moves (new sections, removing content, reordering whole sections, anything that changes scope or intent), present a short prioritized list with line references and apply on the author's go-ahead.

**Pass 4 - Final validation.**
- Code fences balanced, headings/lists/links render, no trailing whitespace, no em dashes, ASCII punctuation throughout.
- Compare the result against at least one existing article.
- Report what you changed and, separately, what you flagged for the author to confirm.

For substantial rewrites, read `references/style-guide.md` before editing. For a quick grammar-only pass, Pass 1 plus the rules below are enough.

## Voice Rules

- Keep the author's first-person framing where it exists: `I tried`, `I ended up`, `I like`, `For me`.
- Use practical walkthrough language: `Let's`, `Now`, `Next`, `For example`, `At this point`, `That's it`.
- Prefer direct claims over abstract framing.
- Keep paragraphs compact. One idea per paragraph is usually enough.
- Use bullets for tradeoffs, requirements, pros/cons, and takeaways.
- Keep technical nouns concrete: name the library, command, file, protocol, container, VM, database, or option.
- Preserve mild personal judgment when it helps: `annoying`, `convenient`, `not so critical`, `reasonable`.
- Let the article sound written by an engineer who tried the thing, not by a product marketer.

## Editing Rules

- Do not over-polish. If a sentence becomes too smooth, formal, or generic, pull it back toward the author's normal article voice.
- Do not replace all `I` and `we` with neutral passive prose.
- Do not add broad claims, marketing language, or invented benefits.
- Do not add explanations of basic concepts unless the draft needs them for the reader to follow the workflow.
- Keep code snippets and commands stable unless they contain obvious typos or placeholders.
- When changing commands, make them more executable and concrete, but do not alter the intended workflow.
- Prefer ASCII punctuation. Do not use em dashes.

**Apply directly vs propose first.** Apply directly: grammar, fluency, consistency, generic-AI fixes, missing links, trimming filler, fixing ordering, de-duplicating, and obvious factual or command typos. Propose first: unverifiable claims, new sections, removing content, reordering whole sections, or anything that changes the article's scope or intent.
