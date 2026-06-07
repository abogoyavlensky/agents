# Style Guide

## Observed Style

The articles are practical technical walkthroughs. They usually start with a concrete problem, explain why common approaches are not enough, then move into commands, config, and small examples.

The voice is direct and personal:

- "Sometimes I need..."
- "In my experience..."
- "I like this approach because..."
- "After evaluating several options, I settled on..."
- "Let's start..."
- "Now we can..."

The writing often uses first-person singular for motivation and first-person plural for walkthrough steps. Keep both when they fit.

## Structure Pattern

Typical article shape:

1. Short motivation or TL;DR.
2. Requirements, constraints, or alternatives.
3. Chosen approach and why.
4. Step-by-step setup with code blocks.
5. Verification or usage example.
6. Caveats, wrapping up, or takeaways.

Use `###` headings for main sections and `####` for detailed setup sections. Keep headings literal and useful: `Setup`, `Workflow`, `Caveats`, `Run formatting`, `Deployment config`.

## Sentence Pattern

Prefer:

- "For the simplest setup, use one of the built-in templates."
- "Now we can fix it:"
- "At this point, you can check the database schema."
- "This approach works, but it has a few downsides."
- "I didn't find a way to set this using only the CLI."

Avoid:

- "This comprehensive solution streamlines the developer experience."
- "The following section delves into the configuration."
- "This robust approach ensures seamless operation."
- "The outcome is a polished and scalable workflow."

## Tone Calibration

Use enough polish to remove mistakes, but not so much that the post stops sounding like the author.

Good:

> I tried a VM per project, but it wasn't convenient because I sometimes want to add more directories to the agent's context.

Too formal:

> A per-project VM model proved suboptimal due to recurring contextual directory requirements.

Good:

> Egress control is still unsolved. You have to figure out how to restrict outgoing HTTP requests yourself: local proxies, `/etc/hosts`, or something else.

Too generic:

> Network egress management remains an area requiring additional operational consideration.

## Common Fixes

- Fix typos and malformed words aggressively.
- Break long tangled sentences into two or three shorter sentences.
- Convert vague abstraction into a specific tool, command, or effect.
- Replace passive constructions when active voice is natural.
- Keep casual transitions if they help the walkthrough flow.
- Preserve links, code samples, shell commands, and project-specific names.
- Keep personal caveats and tradeoffs instead of flattening them into neutral documentation.
