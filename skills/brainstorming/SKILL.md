---
name: brainstorming
description: You MUST use this before any creative work - creating features, building components, adding functionality, or modifying behavior. Explores user intent, requirements and design before implementation.
---

# Brainstorming Ideas Into Plans

Help turn ideas into fully formed designs and implementation plans through natural collaborative dialogue.

Start by understanding the current project context, then ask questions one at a time to refine the idea. Once you understand what you're building, present the design, write the plan, and get user approval.

<HARD-GATE>
Do NOT invoke any implementation skill, write any code, scaffold any project, or take any implementation action until you have presented a design and the user has approved it. This applies to EVERY project regardless of perceived simplicity.
</HARD-GATE>

## Checklist

You MUST create a task for each of these items and complete them in order:

1. **Explore project context** — check files, docs, recent commits
2. **Ask clarifying questions** — one at a time, understand purpose/constraints/success criteria
3. **Propose 2-3 approaches** — with trade-offs and your recommendation
4. **Present design** — in sections scaled to their complexity, get user approval after each section
5. **Write plan document** — follow plan-format-guide.md for structure; save to `docs/plans/000-<topic>-plan.md` and commit
   - The number in front is incremental and for ordering - use 000 for the first plan, then 001, etc
   - add to git: `git add -N docs/plans/000-<topic>-plan.md`
6. **Plan review loop** — dispatch plan-document-reviewer subagent (see plan-document-reviewer-prompt.md) using gpt-5.4 (if available) and xhigh thinking effort with precisely crafted review context (never your session history); fix issues and re-dispatch until approved (max 3 iterations, then surface to human)
7. **User reviews written plan** — ask user to review the plan file before proceeding
8. **Transition to implementation** — invoke /executing-plans skill to implement the plan

## The Process

**Understanding the idea:**

- Check out the current project state first (files, docs, recent commits)
- Before asking detailed questions, assess scope: if the request describes multiple independent subsystems (e.g., "build a platform with chat, file storage, billing, and analytics"), flag this immediately. Don't spend questions refining details of a project that needs to be decomposed first.
- If the project is too large for a single plan, help the user decompose into sub-projects: what are the independent pieces, how do they relate, what order should they be built? Then brainstorm the first sub-project through the normal design flow. Each sub-project gets its own plan -> implementation cycle.
- For appropriately-scoped projects, ask questions one at a time to refine the idea
- Prefer multiple choice questions when possible, but open-ended is fine too
- Only one question per message - if a topic needs more exploration, break it into multiple questions
- Focus on understanding: purpose, constraints, success criteria

**Exploring approaches:**

- Propose 2-3 different approaches with trade-offs
- Present options conversationally with your recommendation and reasoning
- Lead with your recommended option and explain why

**Presenting the design:**

- Once you believe you understand what you're building, present the design
- Scale each section to its complexity: a few sentences if straightforward, up to 200-300 words if nuanced
- Ask after each section whether it looks right so far
- Cover: architecture, components, data flow, error handling, testing
- Be ready to go back and clarify if something doesn't make sense

**Design for isolation and clarity:**

- Break the system into smaller units that each have one clear purpose, communicate through well-defined interfaces, and can be understood and tested independently
- For each unit, you should be able to answer: what does it do, how do you use it, and what does it depend on?
- Can someone understand what a unit does without reading its internals? Can you change the internals without breaking consumers? If not, the boundaries need work.
- Smaller, well-bounded units are also easier for you to work with - you reason better about code you can hold in context at once, and your edits are more reliable when files are focused. When a file grows large, that's often a signal that it's doing too much.

**Working in existing codebases:**

- Explore the current structure before proposing changes. Follow existing patterns.
- Where existing code has problems that affect the work (e.g., a file that's grown too large, unclear boundaries, tangled responsibilities), include targeted improvements as part of the design - the way a good developer improves code they're working in.
- Don't propose unrelated refactoring. Stay focused on what serves the current goal.

## Writing the Plan

After the design is approved, write the implementation plan document.

- Follow the format and structure in plan-format-guide.md
- Include the approved design in the plan — architecture, components, data flow, error handling, testing strategy. The plan should be self-contained so the executor has full context.
- Map out the file structure first — which files will be created or modified
- Break work into bite-sized tasks with clear steps (test, implement, verify, commit)
- Describe what to implement clearly, but don't inline full code — the executor can write code from clear descriptions
- Exact file paths, exact commands, expected outputs
- DRY, YAGNI, TDD, frequent commits

## Plan Review Loop

After writing the plan document:

1. Dispatch plan-document-reviewer subagent (see plan-document-reviewer-prompt.md) using gpt-5.4 (if available) and xhigh thinking effort
2. If Issues Found: fix, re-dispatch, repeat until Approved
3. If loop exceeds 3 iterations, surface to human for guidance

**Review loop guidance:**
- Same agent that wrote the plan fixes it (preserves context)
- Reviewers are advisory — explain disagreements if you believe feedback is incorrect

## User Review Gate

After the plan review loop passes, ask the user to review the written plan before proceeding:

> "Plan written and committed to `<path>`"

Wait for the user's response. If they request changes, make them and re-run the plan review loop. Only proceed once the user approves.

## Execution Handoff

After the user approves the plan, offer execution choice:

**"Plan approved. Want to implement it now?"**

- **If yes:** Use /executing-plans skill
- **If no:** Stop and wait for user to trigger execution later

## Key Principles

- **One question at a time** - Don't overwhelm with multiple questions
- **Multiple choice preferred** - Easier to answer than open-ended when possible
- **YAGNI ruthlessly** - Remove unnecessary features from all designs
- **Explore alternatives** - Always propose 2-3 approaches before settling
- **Incremental validation** - Present design, get approval before moving on
- **Be flexible** - Go back and clarify when something doesn't make sense
