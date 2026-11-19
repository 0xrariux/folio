#!/usr/bin/env python3
"""Extract only the reviewed top changelog entry for the release draft."""
from pathlib import Path
import re
import tomllib
root = Path(__file__).resolve().parents[1]
version = tomllib.loads((root / 'Cargo.toml').read_text())['package']['version']
text = (root / 'CHANGELOG.md').read_text()
sections = re.split(r'^## ', text, flags=re.MULTILINE)[1:]
if not sections or not re.search(r'(?<![\d.])' + re.escape(version) + r'(?![\d.])', sections[0].splitlines()[0]):
    raise SystemExit('Top changelog entry must match the release version')
print('\n'.join(sections[0].splitlines()[1:]).strip())
