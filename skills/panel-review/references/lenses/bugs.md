## Lens: bugs

Hunt for code that does the wrong thing when it runs. Read the changed code and the code that calls
it, since a defect is often only visible from the caller's side.

Look for:

- logic that is inverted, off by one, or wrong at the boundary: empty input, a single element, the last
  iteration, the zero value
- nil, null or missing values dereferenced on a path that can actually produce them
- concurrency: state shared without synchronization, a lock not released on every return, a wait that
  can block forever, background work outliving what it writes to
- resources acquired and not released on the error path: files, connections, timers, subprocesses
- errors dropped, swallowed, or returned without the context needed to act on them
- an error path that leaves state half-updated
- state persisting between calls where the caller assumes it does not
- a signature or return shape changed while a caller still uses the old one

For each candidate, name the input or sequence that triggers it before you report it. If you cannot
produce one, you have found a smell, not a bug. Leave it out.
