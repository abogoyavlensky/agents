---
name: community-post
description: Write developer-community messages about a project - Discord/Slack welcome pins, status updates, release announcements, channel intros - grounded in the repo's actual released vs unreleased state. Use whenever the user wants a welcome message, pinned message, announcement, status update, "what's new" post, or channel intro for a project community, even if they only say "help me write a message about my project". Reads the repo first so the post never advertises what users can't install.
---

# Community Post

Write a message developers will actually read, about a project as it actually is. The two failure modes this skill exists to prevent: posts that read like marketing (skimmed, ignored) and posts that promise features the reader's `brew install` cannot deliver (trust burned on day one).

## Step 1: Ground in the repo before writing a word

The message's credibility comes from matching reality. Check, in the actual repo:

- **What's released**: current branch, latest tag (`git tag`), the version constant, GitHub Releases. Work on feature branches is NOT released, even if merged docs describe it.
- **The install story**: copy the exact install commands from the README (brew tap, mise, curl script, prerequisites like a separate runtime). The try-it snippet is the highest-stakes text in the post - it's the first thing people paste.
- **Claimed capabilities**: if the post says "X works", verify X works in the released version, not on a branch or with a patched dependency. When the user names features or supported libraries, cross-check rather than transcribe - people misremember their own project's status (this is the most common correction in practice).

If a claim can't be verified quickly, ask the user or hedge it explicitly ("in progress") - never silently promote it to released.

## Step 2: Pick the message shape

Ask (or infer from context) two things: **does the audience know the underlying tech?** and **what kind of message is this?**

| Type | Pinned? | Shape |
|---|---|---|
| Welcome / channel intro | yes | pitch → 30-second try-it → links → what to post here (~10 lines) |
| Status update | no | 3-5 bullets of in-flight work, each concrete, with a released/unreleased line at the end |
| Release announcement | no | what shipped, one highlight demo, upgrade command, link to notes |

Pins are reference, updates are news. Keep them separate messages rather than one mega-pin: long pins get skimmed, and news pinned forever goes stale. Offer the second message unpinned.

## Step 3: Write it

- **Lead with the hook, not the category.** "Package manager for X" means nothing to someone who doesn't know X. Say what it gives the reader ("Clojure-flavored CLIs that start instantly, no JVM") and bury the category as a second clause. If the audience already knows the ecosystem, sharpen the hook to the differentiator instead.
- **The try-it snippet converts.** A curious developer is won by a copy-pasteable block that works in under a minute (`install` + `new` + `run`). Put it high. Flag to the user any command you couldn't verify cold - it's the first thing that will fail publicly.
- **Separate shipped from in-flight, explicitly.** A status update that reads like released features harvests "it doesn't work" replies. One closing line ("none of this is in vX.Y yet") is enough.
- **Concrete beats abstract in every bullet.** "Go deps" is abstract; "SQLite in a single static binary" is the same feature as a hook. For each in-flight item, find the user-visible payoff.
- **Invite the right traffic.** End a welcome pin with what to post (questions, bugs, things you built) and an honest maturity note ("pre-1.0, rough edges expected, reports are gold") - it converts complaints into contributions.
- **Anticipate the awkward question.** If the project deliberately doesn't do something adjacent (e.g. no Maven support, by design), don't put it in the post - instead hand the user a ready one-liner answer for when it comes up in the channel.
- Emoji: sparing and structural (one per section as a visual anchor) works on Discord/Slack; skip them for mailing lists or GitHub.
- No em-dashes; use commas, periods, or hyphens. Apply the writing-clearly skill's rules if it is available: active voice, concrete language, omit needless words.

## Step 4: Deliver

- Hand over the final text as raw markdown in a fenced block, ready to paste.
- Warn about platform paste gotchas when they apply (e.g. a code block nested inside the outer fence ends at the first inner ``` on Discord - fine when pasting the content, confusing when copying the whole fenced block).
- Note the judgment calls you made (tone, what you left out, unverified commands) so the user can flip them - the user knows their community; you know the repo.

## Example (shape, not template)

A welcome pin produced by this process:

```markdown
👋 **Welcome to #lgx**

**lgx** is a project manager for [let-go](https://github.com/nooga/let-go), a Clojure dialect that compiles to a single fast-starting static binary (no JVM). lgx gives it the missing tooling: git deps, run/test/build, nREPL, scaffolding, and tasks, in one binary.

Try it in under a minute:
brew install nooga/tap/let-go abogoyavlensky/tap/lgx
lgx new hello && cd hello && lgx run

🔗 Repo: <https://github.com/abogoyavlensky/lgx> · Releases · let-go

Post here: questions, bug reports, things you built, ideas. It's pre-1.0 and moving fast. Rough edges are expected and reports are gold. 🙏
```

Notice what carries it: the hook explains the unknown tech in one clause, the snippet is the exact README commands, the unreleased cross-compilation work is absent (it went in a separate unpinned status update), and the maturity note sets expectations.
