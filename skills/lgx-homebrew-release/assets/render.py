#!/usr/bin/env python3
"""Scaffold the Homebrew release pipeline files into an lgx project.

Renders the templates in ./templates/ (next to this script) into a target
project, substituting the project-identity placeholders. Existing files are
left untouched unless --force is given, so it never clobbers a project's
existing CI. It does NOT touch main.lg or lgx.edn — those are edits to files
that already exist, done by the skill's instructions, not here.

Usage:
  render.py --target DIR --name NAME --owner OWNER --repo REPO \
            --desc "Short description" [--version 0.1.0] [--force]

Derived automatically:
  CLASS     CamelCase of NAME (skl -> Skl, my-tool -> MyTool) for the formula
  HOMEPAGE  https://github.com/OWNER/REPO
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TEMPLATES = os.path.join(HERE, "templates")

# template filename -> (relative dest path, executable?)
LAYOUT = {
    "generate-formula.sh": ("scripts/generate-formula.sh", True),
    "tag.lg": ("scripts/tag.lg", False),
    "checks.yml": (".github/workflows/checks.yml", False),
    "release.yml": (".github/workflows/release.yml", False),
}


def formula_class(name):
    parts = [p for p in re.split(r"[^A-Za-z0-9]+", name) if p]
    return "".join(p[:1].upper() + p[1:] for p in parts)


def render(text, subs):
    for key, val in subs.items():
        text = text.replace("@@%s@@" % key, val)
    return text


def write(path, content, force, base, executable=False, newline=True):
    rel = os.path.relpath(path, base)
    existed = os.path.exists(path)
    if existed and not force:
        print(f"  skip (exists): {rel}")
        return "skip"
    os.makedirs(os.path.dirname(path), exist_ok=True)
    data = content if newline else content.rstrip("\n")
    with open(path, "w") as f:
        f.write(data)
    if executable:
        os.chmod(path, 0o755)
    print(f"  {'overwrote' if existed else 'wrote'}: {rel}")
    return "wrote"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", required=True, help="project root to scaffold into")
    ap.add_argument("--name", required=True, help="binary name (from lgx.edn :targets)")
    ap.add_argument("--owner", required=True, help="GitHub owner/org")
    ap.add_argument("--repo", required=True, help="GitHub repo name")
    ap.add_argument("--desc", required=True, help="one-line description for the formula")
    ap.add_argument("--version", default="0.1.0", help="initial version for resources/VERSION")
    ap.add_argument("--force", action="store_true", help="overwrite existing files")
    args = ap.parse_args()

    subs = {
        "NAME": args.name,
        "OWNER": args.owner,
        "REPO": args.repo,
        "DESC": args.desc,
        "CLASS": formula_class(args.name),
        "HOMEPAGE": f"https://github.com/{args.owner}/{args.repo}",
    }

    print(f"Scaffolding into {args.target}")
    print(f"  name={args.name} class={subs['CLASS']} owner={args.owner} "
          f"repo={args.repo}")

    for tmpl, (rel, is_exe) in LAYOUT.items():
        src = os.path.join(TEMPLATES, tmpl)
        with open(src) as f:
            content = render(f.read(), subs)
        write(os.path.join(args.target, rel), content, args.force,
              args.target, executable=is_exe)

    # resources/VERSION must have NO trailing newline: tag.lg builds the git
    # tag as "v" + its contents, and `--version` prints it verbatim.
    write(os.path.join(args.target, "resources", "VERSION"),
          args.version, args.force, args.target, newline=False)

    print("Done. Next: edit main.lg + lgx.edn per the skill, then verify.")


if __name__ == "__main__":
    main()
