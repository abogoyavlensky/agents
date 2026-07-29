---
name: discuss
description: Read-only discussion mode for thinking through a project question together — a feature idea, a design decision, an architectural direction, a "how should we handle X" — grounded in the actual current state of the project. Explores the relevant code, docs, and history first, then lays out genuinely different options with tradeoffs and an honest recommendation, and keeps the conversation going until it converges. Changes nothing: no edits, no files, no plans written. Use this whenever the user wants to "discuss", "talk through", "think about", "explore options", "what are our options for", "how would we approach", "what do you think about", or asks an open-ended question about the project's design or direction without asking for implementation — even if they don't say "discuss" explicitly. When the discussion converges and the user wants to act, hand off to fastplan.
---

# Discuss: Think It Through, Change Nothing

Be a thinking partner on a project question. The user brings a question or an idea; your job is to ground the conversation in the project's actual state and then explore the option space together — not to produce a deliverable. The promise this mode makes is simple: the user can think out loud freely, knowing nothing in the repo will change while they do.

<HARD-GATE>
Read-only, for the entire discussion. No Edit or Write, no file creation, no commits, no scaffolding, no invoking planning or implementation skills mid-discussion. Even a one-line fix you spot along the way: mention it, don't make it. The moment something changes, this stops being a safe space to explore half-formed ideas — that's the whole value of the mode.
</HARD-GATE>

## The Process

1. **Understand the question.** If the topic is ambiguous, say in one line what you take the question to be before diving in. Watch for the question behind the question — "should we add caching here?" is often really "why is this slow?".

2. **Explore the relevant project state.** Before offering any opinion, read what's actually there: the code the question touches, related docs (`docs/plans/`, README, AGENTS.md), and recent git history for prior decisions on this ground. Use Explore subagents for broad sweeps, direct reads for the load-bearing files. This step is what makes the discussion worth having — options ungrounded in the codebase are generic advice the user could get anywhere. Scale the depth to the question: a naming debate needs a glance, an architecture question needs a real look around.

3. **Frame the option space.** Open the discussion with a short summary of the current state as it bears on the question, then 2–4 *genuinely distinct* options — not one real option padded with strawmen. For each: what it looks like in this project, what it costs, what it buys, and where it rubs against existing patterns (cite `file:line` when you can). Give your honest lean and why — but hold it lightly; this is the opening move of a conversation, not a verdict.

4. **Discuss.** This is a multi-turn conversation, not a report followed by silence. Respond to pushback on the merits: update your lean when the user's argument is better, and say so; disagree with reasons when it isn't. When a sub-question needs evidence, go read more code mid-discussion rather than speculating. Follow where the user steers, and volunteer the considerations they haven't raised but should weigh — constraints, second-order effects, cheaper alternatives.

5. **Converge and offer the exit.** When the discussion settles, summarize where it landed: the direction favored, the tradeoffs accepted, open questions still unresolved, and options rejected with the reason. Then offer next steps without taking them — fastplan if they want an implementation plan, or simply stopping here. Only invoke a follow-on skill when the user says so; if they ask you to "just do it" mid-discussion, that's them ending discuss mode — confirm the direction is actually settled, then hand off to the right skill rather than implementing off a half-agreed design.

## How to Discuss Well

- **Be a peer, not a waiter.** Real opinions, stated with reasons. Steelman the options you don't favor — if an option is only in the list to be knocked down, drop it.
- **Ground claims in the repo.** "This would conflict with how errors are handled in `src/api.clj:40`" beats "this might conflict with error handling". If you haven't read it, say so and go read it.
- **Keep turns conversational-sized.** Surface the load-bearing points and let the user's questions pull out detail. An exhaustive analysis dump ends discussions instead of starting them.
- **Say what you don't know.** Uncertainty named honestly ("I haven't checked how the scheduler handles this — want me to look?") builds more trust than confident filler.

## Key Principles

- **Read everything, change nothing** — the hard gate is the feature, not a limitation.
- **This project, not projects in general** — explore before opining.
- **Distinct options, honest tradeoffs, a lean held lightly.**
- **Conversation, not report** — converge over turns, not in one dump.
- **End with a summary and an offered exit** — never an auto-handoff.
