#!/usr/bin/env python3
"""Read-only weekly intake. Reports upstream changes; never changes pins or publishes."""
import argparse
import datetime
import json
from pathlib import Path
import re
import tomllib
import urllib.request
import urllib.parse

ROOT = Path(__file__).resolve().parents[1]


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "Folio-dependency-watch/1", "Accept": "application/json"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read(4 * 1024 * 1024).decode()


def collect(get=fetch):
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    pin = json.loads((ROOT / "folio-update.json").read_text())["rune_revision"]
    rust = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    turso_spec = cargo["dependencies"]["turso"]
    turso = (turso_spec if isinstance(turso_spec, str) else turso_spec["version"]).lstrip("=")
    rows = []
    sources = [
        ("Rust", rust, "https://static.rust-lang.org/dist/channel-rust-stable.toml", "https://doc.rust-lang.org/releases.html", "Compiler/MSRV, warnings and platform support; rebuild and run the full suite."),
        ("Turso", turso, "https://crates.io/api/v1/crates/turso", "https://github.com/tursodatabase/turso/releases", "Storage/encryption compatibility; verify old profiles and encrypted backups before changing the storage contract."),
        ("Rune", pin, "https://api.github.com/repos/anomalyco/rune", f"https://github.com/anomalyco/rune/compare/{pin}...dev", "UI/API/rendering, drag-and-drop, packaging and vendor patches; compare changes and test Folio interactions."),
    ]
    for name, current, url, notes, impact in sources:
        try:
            if name == "Rune":
                branch = json.loads(get(url))["default_branch"]
                encoded = urllib.parse.quote(branch, safe="")
                url = f"https://api.github.com/repos/anomalyco/rune/commits/{encoded}"
                notes = f"https://github.com/anomalyco/rune/compare/{pin}...{encoded}"
            raw = get(url)
            if name == "Rust":
                upstream = tomllib.loads(raw)["pkg"]["rust"]["version"].split()[0]
            elif name == "Turso":
                upstream = json.loads(raw)["crate"]["max_stable_version"]
            else:
                upstream = json.loads(raw)["sha"]
                if not re.fullmatch("[0-9a-f]{40}", upstream):
                    raise ValueError("Invalid Rune revision")
            row = {"dependency": name, "pinned": current, "upstream": upstream,
                   "status": "current" if upstream == current else "review required",
                   "source": url, "changes": notes, "folio_review": impact}
            try:
                if name == "Rust":
                    detail_url = "https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md"
                    detail = get(detail_url).split("\nVersion ", 2)[0][:12000]
                elif name == "Turso":
                    detail_url = "https://api.github.com/repos/tursodatabase/turso/releases?per_page=10"
                    releases = json.loads(get(detail_url))
                    detail = "\n\n".join(f"{r['tag_name']}\n{r.get('body', '')}" for r in releases)[:12000]
                else:
                    detail_url = f"https://api.github.com/repos/anomalyco/rune/compare/{pin}...{upstream}"
                    comparison = json.loads(get(detail_url))
                    detail = "\n".join(c["commit"]["message"] for c in comparison.get("commits", []))[:8000]
                    row["changed_files"] = [f["filename"] for f in comparison.get("files", [])]
                    row["compare_status"] = comparison.get("status")
                    if comparison.get("status") in ("behind", "diverged"):
                        row["status"] = "divergent history: manual review"
                row["upstream_changes_source"] = detail_url
                row["upstream_changes"] = detail
            except Exception as error:
                row["changelog_error"] = str(error)
                row["status"] = "changelog check failed"
            rows.append(row)
        except Exception as error:
            rows.append({"dependency": name, "pinned": current, "status": "check failed", "error": str(error), "source": url, "changes": notes})
    return {"checked_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "folio_version": cargo["package"]["version"], "dependencies": rows, "publication": "Manual approval required. This report does not upgrade or publish anything."}


def markdown(report):
    lines = [f"# Folio dependency review — {report['checked_at'][:10]}", "", report["publication"], "", "| Dependency | Pinned | Upstream | Status |", "|---|---|---|---|"]
    for row in report["dependencies"]:
        lines.append(f"| {row['dependency']} | `{row['pinned']}` | `{row.get('upstream', 'unavailable')}` | {row['status']} |")
    for row in report["dependencies"]:
        lines += ["", f"## {row['dependency']}", "", f"[Upstream changes]({row['changes']})", "", row.get("folio_review", row.get("error", ""))]
        if row.get("upstream_changes"):
            # JSON keeps the complete bounded excerpt; Markdown stays readable.
            excerpt = row["upstream_changes"][:3000].replace("</details>", "")
            lines += ["", "<details><summary>Collected upstream notes / commits</summary>", "", excerpt, "", "</details>"]
        if row.get("changelog_error"):
            lines += ["", "Changelog unavailable: " + row["changelog_error"]]
    lines += ["", "Nexus must retrieve the detailed changelog/diff, assess the actual Folio call sites, prepare an isolated candidate and attach its validation evidence before approval."]
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "release-artifacts/dependency-watch")
    args = parser.parse_args()
    report = collect()
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    (args.output / "report.md").write_text(markdown(report))
    print(markdown(report))
    if any(row["status"] in ("check failed", "changelog check failed") for row in report["dependencies"]):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
