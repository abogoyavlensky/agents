## Lens: security

Catch the obvious mistakes that would not survive review at any company. This is not a full security
audit, so do not pad it with theory: every finding names the untrusted input and where it lands.

Look for:

- injection: shell, SQL, HTML, template, log. Anywhere external input lands in a string that is then
  interpreted
- path traversal: a user-supplied path joined to a base directory and read or written without checking
  where it resolves
- untrusted deserialization: `eval`, `pickle`, `Marshal`, YAML loaders that build objects, on input an
  attacker controls
- hardcoded secrets, tokens or credentials in source, config or tests
- missing validation at a trust boundary: HTTP handlers, CLI arguments from external sources, file
  uploads, webhook payloads, subprocess output
- authorization checked on one path to an action but not another
- catastrophic regex: unbounded backtracking on user input
- secrets or personal data written to logs or error messages

Validation belongs at the system edge. Internal code calling internal code is a trusted boundary:
do not ask for validation between components that already trust each other. The project profile says
what is actually exposed. A CLI that only its owner runs on their own files has a much smaller surface
than a server.
