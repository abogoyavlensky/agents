---
name: liteplan
description: Use this before small or medium creative coding work - creating features, building components, adding functionality, or modifying behavior. Explores user intent, requirements, and design, then writes a lightweight implementation plan compatible with executing-plans.
---

# Liteplan: Turning Ideas Into Lightweight Plans

## Overview

Help turn small ideas into fully formed designs and implementation-ready
plans through natural collaborative dialogue.

Start by understanding the current project context, then ask questions one
at a time to refine the idea. Once you understand what you're building,
present the design in small sections scaled to the work, checking after
each section whether it looks right so far.

<HARD-GATE>
Do NOT invoke any implementation skill, write any code, scaffold any project, or take any implementation action until you have presented a design and the user has approved it. This applies to EVERY project regardless of perceived simplicity.
</HARD-GATE>

## The Process

**Understanding the idea:**
- Check out the current project state first (files, docs, recent commits)
- Ask questions one at a time to refine the idea
- Prefer multiple choice questions when possible, but open-ended is fine too
- Only one question per message - if a topic needs more exploration, break it into multiple questions
- Focus on understanding: purpose, constraints, success criteria

**Exploring approaches:**
- Propose 2-3 different approaches with trade-offs
- Present options conversationally with your recommendation and reasoning
- Lead with your recommended option and explain why

**Presenting the design:**
- Once you believe you understand what you're building, present the design
- Break it into sections scaled to their complexity: a few sentences for
  straightforward work, up to 200-300 words if nuanced
- Ask after each section whether it looks right so far
- Cover: architecture, components, data flow, error handling, testing
- Be ready to go back and clarify if something doesn't make sense

## After the Design

**Documentation:**
- Write the approved design and executable task list to
  `docs/plans/YYYY-MM-DD-<topic>.md`
- Use `../brainstorming/plan-format-guide.md` as the reference for the
  document format, but keep the plan lightweight
- Use /writing-clearly skill
- First, commit the plan document to git without attribution with one-line
  message

## Plan Document Format

Liteplan documents must be compatible with `executing-plans`. Use the
same core output contract as the brainstorming plan format, without the
brainstorming review loop.

Every plan starts with:

```markdown
# [Feature Name] Implementation Plan

> **For agentic workers:** Use executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** [One sentence describing what this builds]

**Tech Stack:** [Key technologies/libraries]

---
```

Then include these sections:

- `## Design` - approved design context. Cover only the parts relevant to
  the small task: architecture, components, data flow, error handling, and
  testing strategy as needed.
- `## File Structure` - exact files to create or modify, with each file's
  responsibility. Keep this brief when only one or two files are involved.
- `## Implementation Steps` - bite-sized `### Task N:` sections with
  checkbox steps that `executing-plans` can mark complete.

Default task shape:

```markdown
### Task N: [Specific task name]

**Files:**
- Create: `exact/path/to/new-file.ext`
- Modify: `exact/path/to/existing-file.ext`
- Test: `exact/path/to/test-file.ext`

- [ ] **Step 1: Write or update the focused test**
  Describe expected behavior.

- [ ] **Step 2: Run the focused test**
  Run: `<exact test command>`
  Expected: `<expected result>`

- [ ] **Step 3: Implement the change**
  Describe the minimal implementation and key constraints.

- [ ] **Step 4: Run verification**
  Run: `<exact verification command>`
  Expected: `<expected result>`
```

For docs-only, config-only, or wiring-only work, skip irrelevant TDD
steps, but still include exact files, checkbox steps, and verification.
Do not inline full code in the plan; describe the change clearly enough
for an executor to implement it.

**"Plan approved. Want to implement it now?"**

- **If yes:** Use /executing-plans skill
- **If no:** Stop and wait for user to trigger execution later

## Key Principles

- **One question at a time** - Don't overwhelm with multiple questions
- **Multiple choice preferred** - Easier to answer than open-ended when possible
- **YAGNI ruthlessly** - Remove unnecessary features from all designs
- **Explore alternatives** - Always propose 2-3 approaches before settling
- **Incremental validation** - Present design in sections, validate each
- **Be flexible** - Go back and clarify when something doesn't make sense
