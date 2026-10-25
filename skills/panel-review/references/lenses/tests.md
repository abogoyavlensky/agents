## Lens: tests

Review the tests, not the production code behind them. Read a neighbouring test file first, so what you
call a convention is this project's convention and not yours.

A missing test is a finding only when you can name the defect it would catch. "This has no test" is not
a finding; "nothing would catch that an empty input returns the previous result" is. The exception is a
project that documents a rule requiring tests for new behaviour: then the rule alone is enough.

Look for:

- a fix with no test that fails without it, so the defect just repaired can come back unnoticed
- a test that would still pass if the code it covers were deleted or inverted: assertions on values the
  test supplied itself, on a stub it configured, or behind a condition that quietly skips them
- ignored failures and disabled cases that make a test green by construction, and a skip whose stated
  reason no longer holds
- state shared between tests, an assumption about run order, or a sleep standing in for synchronization
- assertions on which internal calls happened where the observable result is what matters
- names and structure that fight the project's own: a generic test name, several scenarios crammed
  into one body where the neighbours split them
- one uncovered path that is reachable in real use, fails in a way someone notices, and whose symptom
  you can state. Never an inventory of uncovered branches

When a test is genuinely missing, report it once, naming the defect and where the test belongs.
