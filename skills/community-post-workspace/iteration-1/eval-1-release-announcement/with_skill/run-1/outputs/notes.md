# Judgment calls for the wtr v0.2.2 announcement

## Grounding (what I verified in the repo)

- HEAD of master == tag `v0.2.2` == `origin/master`, and `resources/VERSION` says `0.2.2`, so the announcement describes exactly what is released.
- v0.2.2 contains a single commit ("Bump tiny-cli and wtr"): it bumps the tiny-cli dependency v0.2.1 -> v0.2.2 and adjusts wtr's completion tests. The entire user-visible change therefore comes from tiny-cli v0.2.2, which I read in the local checkout at /Users/andrew/Projects/tiny-cli: "Options can go after args" (#5) - options and positionals interleave freely on non-variadic commands, both in parsing and in shell completion, with variadic commands (wtr's `run`) keeping the old "first positional ends option parsing" behavior by design.
- The "no need to regenerate completion scripts" claim is verified: the generated bash/zsh/fish scripts shell out to `wtr __complete`, so the new candidates come from the upgraded binary.
- The GitHub release v0.2.2 exists but has an auto-generated body (just a compare link), so the announcement carries the substance itself and links to the release page.

## Framing choices

- Framed it as a small quality-of-life release rather than padding it out. It is a one-commit dependency bump; overselling it would read as marketing. The concrete before/after example (`wtr remove feature-x --force`) is the hook.
- Included the `wtr run` exception explicitly. It is the one place a reader might expect the new behavior and not get it, and the exception is actually the feature there (args flow to the child command). Better to preempt the "why doesn't run do this" question in the post.
- Left out the tiny-cli "flush help/version output on JVM/Babashka" fix: it does not apply to the wtr binary (let-go runtime), so it is not a wtr change.
- Did not mention wtr v0.2.1's `create --sh` feature; this is a v0.2.2 announcement, not a catch-up post. Say the word if you want a combined "what's new since v0.2.0" instead.
- Assumed the #wtr audience already knows what wtr is, so there is no product pitch or install-from-scratch block, just the upgrade command. Emoji kept to two structural anchors, per Discord conventions.

## Unverified commands (flagging per the skill)

- `brew upgrade abogoyavlensky/tap/wtr` - not run cold here (no Homebrew in this environment). The tap name matches the README's install command, and a previous commit message ("Bump version to check if brew formula works for upgrading") suggests the upgrade path has been exercised, but I could not verify the bottle for v0.2.2 is already published. Worth a 10-second check before posting.
- `mise up` - standard mise upgrade for `github:...@latest` tools, but likewise not executed here.

## Paste gotcha

The message contains fenced code blocks. If you copy it from a chat window where I show it inside an outer fence, Discord will end the outer fence at the first inner ```. Paste the file content itself (release-announcement.md) and it renders fine.
