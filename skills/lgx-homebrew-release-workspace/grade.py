#!/usr/bin/env python3
"""Programmatic grader for the lgx-homebrew-release eval.

For a run directory (containing project/ and outputs/), check each assertion
for the given eval and write grading.json with fields the viewer expects
(expectations[].text/passed/evidence + summary).

Usage: grade.py <run-dir> <eval-id>
"""
import json
import os
import re
import sys


def read(p):
    try:
        with open(p, "rb") as f:
            return f.read()
    except OSError:
        return None


def text(p):
    b = read(p)
    return b.decode("utf-8", "replace") if b is not None else None


def check_exists(proj, rel):
    p = os.path.join(proj, rel)
    return (os.path.exists(p), f"{rel} {'exists' if os.path.exists(p) else 'MISSING'}")


def check_exec(proj, rel):
    p = os.path.join(proj, rel)
    ok = os.path.exists(p) and os.access(p, os.X_OK)
    return (ok, f"{rel} {'is executable' if ok else 'not executable / missing'}")


def check_contains(proj, rel, needle, label=None):
    t = text(os.path.join(proj, rel))
    if t is None:
        return (False, f"{rel} MISSING (looking for {label or needle!r})")
    ok = (re.search(needle, t, re.MULTILINE) is not None) if label else (needle in t)
    where = label or repr(needle)
    return (ok, f"{rel} {'contains' if ok else 'does NOT contain'} {where}")


def check_version_no_newline(proj):
    b = read(os.path.join(proj, "resources/VERSION"))
    if b is None:
        return (False, "resources/VERSION MISSING")
    s = b.decode("utf-8", "replace")
    ok = s == "0.1.0"
    return (ok, f"resources/VERSION bytes={b!r} (want b'0.1.0', no trailing newline)")


def check_no_placeholders(proj):
    hits = []
    for root, _dirs, files in os.walk(proj):
        if os.sep + ".git" in root:
            continue
        for fn in files:
            p = os.path.join(root, fn)
            t = text(p)
            if t and "@@" in t:
                hits.append(os.path.relpath(p, proj))
    return (not hits, "no @@ placeholders" if not hits else f"placeholders left in: {hits}")


def summary_mentions(run_dir, *needles):
    """Check the executor's summary.md (or final message) for manual-step mentions."""
    blob = ""
    out = os.path.join(run_dir, "outputs", "summary.md")
    t = text(out)
    if t:
        blob += t
    ok = all(re.search(n, blob, re.I) for n in needles)
    present = [n for n in needles if re.search(n, blob, re.I)]
    return (ok, f"summary.md mentions {present} of {list(needles)}")


def grade(run_dir, eval_id):
    proj = os.path.join(run_dir, "project")
    checks = []

    def add(text_, result):
        passed, evidence = result
        checks.append({"text": text_, "passed": bool(passed), "evidence": evidence})

    if eval_id == 1:
        add("scripts/generate-formula.sh exists and is executable",
            check_exec(proj, "scripts/generate-formula.sh"))
        add("generate-formula.sh declares 'class Gizmo' and bin.install \"gizmo\"",
            (all(x in (text(os.path.join(proj, "scripts/generate-formula.sh")) or "")
                 for x in ["class Gizmo", 'bin.install "gizmo"']),
             "checked class Gizmo + bin.install \"gizmo\""))
        add("generate-formula.sh points at github.com/acme/gizmo",
            check_contains(proj, "scripts/generate-formula.sh", "acme/gizmo"))
        add("scripts/tag.lg exists and reads resources/VERSION",
            (os.path.exists(os.path.join(proj, "scripts/tag.lg"))
             and 'VERSION' in (text(os.path.join(proj, "scripts/tag.lg")) or ""),
             "tag.lg present + references VERSION"))
        add("release.yml contains a 'homebrew:' job",
            check_contains(proj, ".github/workflows/release.yml", r"^\s*homebrew:", "homebrew: job"))
        add("release.yml pushes Formula/gizmo.rb to acme/homebrew-tap",
            (all(x in (text(os.path.join(proj, ".github/workflows/release.yml")) or "")
                 for x in ["Formula/gizmo.rb", "acme/homebrew-tap"]),
             "checked Formula/gizmo.rb + acme/homebrew-tap"))
        add("release.yml smoke-tests with 'gizmo --version'",
            check_contains(proj, ".github/workflows/release.yml", r"gizmo --version", "gizmo --version"))
        add(".github/workflows/checks.yml exists",
            check_exists(proj, ".github/workflows/checks.yml"))
        add("resources/VERSION is exactly '0.1.0' with no trailing newline",
            check_version_no_newline(proj))
        add("main.lg reads the version via (io/resource \"VERSION\")",
            (("io/resource" in (text(os.path.join(proj, "main.lg")) or ""))
             and ("VERSION" in (text(os.path.join(proj, "main.lg")) or "")),
             "main.lg references io/resource + VERSION"))
        add("lgx.edn has :resource-paths and a release task running scripts/tag.lg",
            (all(x in (text(os.path.join(proj, "lgx.edn")) or "")
                 for x in [":resource-paths", "tag.lg"]),
             "checked :resource-paths + tag.lg in lgx.edn"))
        add("README has 'brew install acme/tap/gizmo'",
            check_contains(proj, "README.md", "brew install acme/tap/gizmo"))
        add("No @@ placeholders remain", check_no_placeholders(proj))
        add("Summary surfaces homebrew-tap repo + HOMEBREW_TAP_TOKEN",
            summary_mentions(run_dir, "homebrew-tap", "HOMEBREW_TAP_TOKEN"))

    elif eval_id == 2:
        add("Pre-existing checks.yml preserved ('# gizmo project CI - keep me')",
            check_contains(proj, ".github/workflows/checks.yml", "gizmo project CI - keep me"))
        add("generate-formula.sh exists (executable) with class Gizmo + bin.install gizmo",
            (os.access(os.path.join(proj, "scripts/generate-formula.sh"), os.X_OK)
             and all(x in (text(os.path.join(proj, "scripts/generate-formula.sh")) or "")
                     for x in ["class Gizmo", 'bin.install "gizmo"']),
             "exec + class Gizmo + bin.install \"gizmo\""))
        add("scripts/tag.lg exists", check_exists(proj, "scripts/tag.lg"))
        add("release.yml has a 'homebrew:' job pushing Formula/gizmo.rb",
            (all(x in (text(os.path.join(proj, ".github/workflows/release.yml")) or "")
                 for x in ["homebrew:", "Formula/gizmo.rb"]),
             "checked homebrew: job + Formula/gizmo.rb"))
        add("release.yml checks job references ./.github/workflows/checks.yml",
            check_contains(proj, ".github/workflows/release.yml", "./.github/workflows/checks.yml"))
        add("resources/VERSION is exactly '0.1.0' with no trailing newline",
            check_version_no_newline(proj))
        add("No @@ placeholders remain", check_no_placeholders(proj))

    passed = sum(1 for c in checks if c["passed"])
    total = len(checks)
    result = {
        "expectations": checks,
        "summary": {"passed": passed, "failed": total - passed, "total": total,
                    "pass_rate": round(passed / total, 3) if total else 0.0},
    }
    out = os.path.join(run_dir, "grading.json")
    with open(out, "w") as f:
        json.dump(result, f, indent=2)
    print(f"{run_dir}: {passed}/{total} ({result['summary']['pass_rate']})")
    return result


if __name__ == "__main__":
    grade(sys.argv[1], int(sys.argv[2]))
