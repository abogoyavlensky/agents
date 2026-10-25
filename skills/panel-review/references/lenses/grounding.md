## Lens: grounding

Check whether what the document claims about the repository is true today. A plan written against code
that is not the way it says fails on the first task, and is the most common way plans break.

Open the code. Every claim you report on cites a file and line, or says plainly that you looked and
found nothing.

- does every file, directory, function, type, signature, flag and command the document names exist,
  where and as it says?
- does the existing behaviour it describes actually happen? Trace the path that produces it
- is something it proposes to build already there, under another name, flag or command?
- does the approach it proposes fit how the code is actually structured? Name what would have to move
  first if it does not
- does it contradict a rule in the project's own docs (AGENTS.md, CLAUDE.md, docs/), or an earlier
  decision recorded in docs/plans/ or git history?

Report what you established, not what you infer from it. "There is no `parse_config` in `src/`" and
"the plan is wrong" are different claims, and only the first is yours to make from reading.

**If you cannot read the code, report that first, as a finding of its own**, naming the command that
failed and what it printed. Every other claim in this lens depends on having opened the files, and a
panel that hears nothing from you would take it to mean the plan checks out.
