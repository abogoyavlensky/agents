---
name: tech-article-writing
description: Edit or draft technical articles, blog posts, tutorials, README-style walkthroughs, and documentation in Andrey Bogoyavlensky's practical technical writing voice. Use when the user asks to improve grammar, fluency, structure, or style of technical prose; align a draft with "my style"; write or rewrite articles for bogoyavlensky.com; or preserve the author's existing voice while applying clear-writing rules.
---

# Tech Article Writing

## Core Approach

Make technical writing clear, practical, and human without turning it into corporate or generic AI prose.

Use this skill together with general clarity rules from `writing-clearly` when available. This skill controls voice and article shape; `writing-clearly` controls grammar, concision, active voice, and avoidance of puffy wording.

## Workflow

1. Read the target draft first.
2. If working inside a repo with existing articles, sample 2-4 nearby articles before editing.
3. Preserve the author's technical intent, examples, links, and command flow.
4. Fix grammar, spelling, punctuation, and awkward phrasing.
5. Keep the style practical: short paragraphs, direct transitions, first-person context, and command-driven walkthroughs.
6. After editing, do a final pass for leftover typos, Markdown/code fence issues, trailing whitespace, and generic AI phrasing.

For substantial rewrites, read `references/style-guide.md` before editing. For small grammar fixes, the rules below are enough.

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

## Final Check

Before finishing:

- Compare the result against at least one existing article when possible.
- Check for spelling and grammar issues.
- Check that headings, lists, links, and code fences still render correctly.
- Check for generic AI wording such as `seamless`, `robust`, `leverage`, `delve`, `groundbreaking`, and similar filler.
- Report what changed briefly and mention any validation performed.
