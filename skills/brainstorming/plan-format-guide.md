# Plan Document Format Guide

Reference for structuring implementation plan documents.

## Plan Document Header

Every plan MUST start with this header:

```markdown
# [Feature Name] Implementation Plan

> **For agentic workers:** Use executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** [One sentence describing what this builds]

**Tech Stack:** [Key technologies/libraries]

---
```

## Design

After the header, include the approved design from the brainstorming discussion. This is the full context the executor needs to understand *what* they're building and *why*, before they see the step-by-step tasks.

Cover the sections that were discussed and approved: architecture, components, data flow, error handling, testing strategy — scaled to their complexity. This should read as a standalone design document, not a transcript of the conversation.

## File Structure

Before defining tasks, map out which files will be created or modified and what each one is responsible for. This is where decomposition decisions get locked in.

- Design units with clear boundaries and well-defined interfaces. Each file should have one clear responsibility.
- Prefer smaller, focused files over large ones that do too much.
- Files that change together should live together. Split by responsibility, not by technical layer.
- In existing codebases, follow established patterns.

## Task Structure

Default pattern (TDD). For tasks without tests (config, docs, wiring), skip test steps.

````markdown
### Task N: [Component Name]

**Files:**
- Create: `exact/path/to/file.py`
- Modify: `exact/path/to/existing.py`
- Test: `tests/exact/path/to/test.py`

- [ ] **Step 1: Write the failing test**
  Description of what to test and expected behavior.

- [ ] **Step 2: Run test to verify it fails**
  Run: `<exact test command>`
  Expected: FAIL with `<expected error>`

- [ ] **Step 3: Write minimal implementation**
  Description of what to implement and key decisions.

- [ ] **Step 4: Run test to verify it passes**
  Run: `<exact test command>`
  Expected: PASS

- [ ] **Step 5: Commit**
  `git commit -m "feat: add specific feature"`
````

## Task Granularity

Each step is one action (2-5 minutes):
- "Write the failing test" — step
- "Run it to make sure it fails" — step
- "Implement the minimal code to make the test pass" — step
- "Run the tests and make sure they pass" — step
- "Commit" — step

## Conventions

- Exact file paths always
- Describe what to implement clearly, but don't inline full code — the executor can write code from clear descriptions
- Exact commands with expected output
- Reference relevant skills with `/` syntax
- DRY, YAGNI, TDD, frequent commits
