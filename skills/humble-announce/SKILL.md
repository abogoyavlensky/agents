---
name: humble-announce
description: Write short, humble, first-person messages about the user's own projects (clj-pulse, lgx, the Clojure Pulse extension, or any repo they maintain) for a small developer channel - a Discord, Slack or Telegram group of a few dozen people, a friends' chat, a niche forum thread. Three kinds - a first announcement ("try it"), a progress update ("what changed since last time"), and a dev-journey note ("what I'm working on or learned"). Use this whenever the user asks to announce, pre-announce, share, post an update, "tell people about" a project or something they shipped, or says "help me write a post/message about X" - even when they don't say "humble" or "announcement". Prefer it over community-post for anything in the user's own voice; leave channel pins and formal release notes to community-post. Grounds every claim in the repo's released state before writing.
---

# Humble announce

The user has a voice for these messages: plain, first person, modest, concrete,
and short. A reader should feel a person telling friends about something they
made, not a product being launched at them. The message in
`references/example.md` is the calibration point; read it before writing, and
aim for that register rather than copying its sentences.

## 1. Ground in the repo before writing

Everything in the message must be true of what a reader can install today.

- Latest tag and version (`git tag --sort=-v:refname | head -1`, the version
  constant). Merged work that is not in a release is not released.
- Install commands, copied verbatim from the README. This block is the first
  thing people paste, so never retype it from memory.
- What it does: the README highlights and feature list.
- What it does not do: the README's support, status, or limitations section.
  This feeds the maturity line.
- Any qualitative performance claim ("within a couple of seconds",
  "lightweight in memory") must map to a recorded number in the repo's docs.
  If there is no number, do not make the claim.

When something cannot be verified quickly, link to it or leave it out. Never
promote it to a fact.

## 2. Take the user's hints

The user may give an audience or channel ("friends on Discord", "the Clojure
Slack", "~20 people who know the ecosystem") and a list of things to leave
out. Both override the defaults below. If the audience already knows the
ecosystem, skip explaining the base technology; if not, one clause is enough.

## 3. Pick the kind

Three kinds share the voice and differ in shape. Infer from the request; ask
only if it is genuinely unclear.

- **First announcement**: the project is new to this audience. Full shape
  below (section 4).
- **Update**: they have seen it before; this is what changed. Shape in
  section 5.
- **Dev-journey note**: no release to point at; a thing learned, a decision
  made, a problem being chewed on. Shape in section 6.

## 4. First announcement

About 10 to 14 lines, in this order:

1. **Hook.** One sentence, first person, saying what it is and why they are
   hearing about it now: "I've been building X, a Y, and it's at a point where
   I'd like a few more people to try it." Not a slogan, not a category label.
2. **What it does.** One paragraph. Lead with the strongest practical property
   stated qualitatively, then the everyday feature set as a plain list in a
   sentence, then one or two things that are distinctive. Plain sentences,
   the kind the user would say out loud.
3. **Install.** A label line, then the README's commands as they are. Two
   alternatives at most.
4. **Links.** The setup or getting-started doc, then the repo. Wrap URLs in
   `<...>` for Discord so they do not unfurl into cards.
5. **Maturity line.** The version, "so expect gaps", and one or two honest
   limitations from the README in parentheses.
6. **The ask.** Invite people to try it on a real project and report what is
   wrong, slow, or missing.

One emoji at the start and one at the end, none in between.

## 5. Update

About 6 to 10 lines. The reader already knows what the project is, so no
hook re-introducing it and no install block unless the install changed.

1. **What changed, in one line**: "clj-pulse 0.6 is out" or "small update on
   lgx", then the version if there is one.
2. **Two to four changes** as plain sentences, each with the user-visible
   payoff, not the implementation. Keep the ones a reader of this channel
   would notice; the rest is in the release notes link. Take them from the
   changelog, release notes, or merged PR titles since the last tag, and
   check each shipped in the version named.
3. **How to get it**: the upgrade command from the README, or "already in the
   extension" when that is the case.
4. **Link** to the release notes or changelog.
5. **An honest line** when there is one: a regression fixed, a thing still
   missing, a behaviour change that could surprise.
6. **The ask**, shorter than the first announcement's: try it, tell me.

One emoji at the start is enough; skip the closing one if the message is
short.

**Tiny update.** A patch release or a single fix does not need the shape
above. Three sentences and a link: what is out, what it fixes (naming the
exact commands or behaviours that broke, so the people it bit recognise it),
how to upgrade. No emoji, no ask; the link on its own line. The lgx 0.3.2
message in `references/example.md` is the calibration point for this size.

## 6. Dev-journey note

About 5 to 10 lines. No version, no install, no ask for testers.

1. **What happened**, first person, one or two sentences: what was tried,
   found, or decided.
2. **Why it mattered**, concretely: the number that moved, the bug that
   explained itself, the design that got simpler. One qualitative claim per
   number, same grounding rule as always.
3. **What is next or still open**, one sentence, honest about uncertainty.
4. Optionally **a question** for the channel if there is a real one.

Link to the commit, PR, or doc when the story lives there. One emoji or
none.

## 7. What to keep out

These are the corrections the user made while arriving at the reference
message; each one exists because the alternative read as marketing.

- No differentiator framing to open a paragraph: not "The one thing it does
  differently", not "What makes it special". Say what it does.
- No competitors by name, no benchmark corpus by name, no comparison tables,
  no "X times faster". A qualitative claim backed by a recorded number is the
  ceiling.
- No editor or client named in the body. Link the editor-setup doc instead, so
  nobody feels excluded and the message does not read as an editor plugin ad.
- No hype words, no exclamation marks, no feature-bullet lists.
- No em-dashes. Short sentences.

## 8. Deliver

Hand over the message as one fenced markdown block, ready to paste. Below it,
a short list of judgment calls: what was left out and why, any install
command that assumes a platform, any claim that sits at the edge of what the
docs record. Add the Discord paste note when the target is Discord: paste the
content, not the outer fence, or the inner lines render oddly.

Then stop. The user knows their audience; offer to adjust tone or trim, but do
not append a second message or a pin unless asked.
