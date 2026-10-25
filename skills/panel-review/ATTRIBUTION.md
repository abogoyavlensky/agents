# Attribution

Parts of this skill's prompt text are adapted from revmux (https://github.com/umputun/revmux):

| file here | adapted from (revmux) |
|---|---|
| `references/preamble-code.md` | `app/prompt/defaults/prompts/profiles/comprehensive.md`, `app/finding/finder-schema.json` (confidence rubric) |
| `references/merge.md` | `app/prompt/defaults/prompts/synthesis.md` |
| `references/verify.md` | `app/prompt/defaults/prompts/verify.md` |
| `references/report.md` | `.claude-plugin/skills/revmux/references/present.md` |
| `references/review-profile-template.md` | `.revmux/profile.md` |
| `references/lenses/{bugs,impl,adversarial,quality,docs,tests,grounding}.md` | `app/prompt/defaults/lenses/` (`docs` also folds in `comments`) |
| prior-round guard in `SKILL.md` | `app/prompt/compose.go` |

The text was trimmed and reworded, and nothing here depends on the revmux binary.

```
MIT License

Copyright (c) 2026 Umputun

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
