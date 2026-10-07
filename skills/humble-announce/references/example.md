# Reference messages

## First announcement

Written for a public Discord channel of about twenty people who know Clojure.
The user's verdict: "almost exactly as I would write it." Use it to calibrate
register, length, and shape. Do not reuse its sentences for another project.

```markdown
👋 I've been building a Clojure language server in Rust, **clj-pulse**, and it's at a point where I'd like a few more people to try it.

It starts fast: on a large project you can jump to definitions within a couple of seconds of opening it, dependencies included, and it stays lightweight in memory. Beyond that it's the everyday set: completion with auto-require, hover, references, rename, symbol search, built-in lints, and clj-kondo on top if you have it installed. It also knows Integrant (config key → `ig/init-key`) and monorepos with several deps.edn projects.

Install:
brew install abogoyavlensky/tap/clj-pulse
or
mise use -g github:abogoyavlensky/clj-pulse

Editor setup for VS Code, Neovim and Zed: <https://github.com/abogoyavlensky/clj-pulse/blob/master/docs/EDITORS.md>
Repo: <https://github.com/abogoyavlensky/clj-pulse>

It's 0.5.x, so expect gaps (ClojureScript and Java support are partial). If you open a real project with it and something is wrong, slow, or missing, I'd really like to hear about it. 🙏
```

## How the first announcement maps to its shape

- Hook: what it is (Clojure language server, Rust) and why now (wants more
  people to try it). No slogan.
- What it does: the fast-start claim first, stated qualitatively ("within a
  couple of seconds", "lightweight in memory"), each backed by a table in the
  repo's PERFORMANCE.md. Then the everyday set in one sentence. Then two
  distinctive things (Integrant, monorepos).
- Install: the README's two commands, unchanged.
- Links: setup doc, then repo, both wrapped in `<...>`.
- Maturity: version line plus two gaps taken from the README's support notes.
- Ask: real project, "wrong, slow, or missing".

## What earlier drafts did that the user rejected

- Opened the second paragraph with "The one thing it does differently: ...".
- Named the comparison server and quoted its numbers next to ours.
- Named the benchmark project (Metabase) and its file count.
- Named one editor in the install instructions and gave its settings key.
- Rounded numbers in the body ("~2 s, ~4 s, ~400 MB") instead of qualitative
  wording plus a link.

## Release update

The user's preferred shape for a usual release. An earlier draft told the
same changes in three prose paragraphs with a closing ask; the user asked to
make it "short and focused": headline, then the tagline from the repo, then a
few words on the release target, then the outlined changes. They liked the
result and asked for it as the standard.

```markdown
🛠️ clj-pulse 0.5.5 is out.
A fast-starting, low-memory Clojure language server.

This release is about ClojureScript and more accurate references and rename.

- `.cljs` files resolve core names through `cljs.core`
- `:require-macros` and `:refer-macros` are understood
- a namespace with both `.clj` and `.cljs` files keeps both, and navigation picks the one matching your file
- keywords in quoted data count as uses
- `->User` and `User.` calls are references to the record
- method params in `defrecord`, `deftype`, `reify` and `extend-protocol` are locals, so rename stays inside the method

Upgrade with `brew upgrade clj-pulse` or `mise up`.
Release notes: <https://github.com/abogoyavlensky/clj-pulse/releases/tag/v0.5.5>
```

What carries it: the tagline sits right under the headline with no blank
line, so the two read as one header. The target line names the theme of the
release, and each list item is a behaviour a user would notice, with the code
they would type. Five merged PRs became six lines; a test-only PR and a
benchmark PR were left out, and niche fixes (`:lint-as` to `declare`,
`defmulti` next to `defmethod`) stayed in the release notes.

## Tiny update

A patch release, written by the user for the same kind of channel. Three
sentences and a link; nothing else. Note what carries it: the regression is
named by the exact commands that broke, so anyone it bit recognises their
bug in one read, and the upgrade commands are the two the README gives.

```markdown
lgx 0.3.2 is out. It fixes a 0.3.1 regression where `lgx repl` and `lgx nrepl` showed no prompt and `lgx run` programs saw no terminal. Upgrade with `brew upgrade lgx` or `mise up`.

https://github.com/abogoyavlensky/lgx/releases/tag/v0.3.2
```

The link is bare rather than `<...>`-wrapped because here the release-page
unfurl is the content; wrap it when the message has its own body.
