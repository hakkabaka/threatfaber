#!/usr/bin/env bash

set -euo pipefail

version="${1:?version is required}"
changelog_path="${2:-CHANGELOG.md}"

if [[ ! -f "$changelog_path" ]]; then
  echo "Missing changelog: $changelog_path" >&2
  exit 1
fi

python3 - "$version" "$changelog_path" <<'PY'
import pathlib
import re
import sys

version = sys.argv[1]
changelog_path = pathlib.Path(sys.argv[2])
content = changelog_path.read_text(encoding="utf-8")
lines = content.splitlines()

heading = re.compile(r"^##\s+\[?([^\]\s]+)\]?(?:\s+-\s+.*)?$")
target_index = None

for index, line in enumerate(lines):
    match = heading.match(line.strip())
    if match and match.group(1).lstrip("v") == version.lstrip("v"):
        target_index = index
        break

if target_index is None:
    print(
        f"Could not find CHANGELOG.md section for version {version}. "
        "Expected a heading like '## [0.1.0]' or '## 0.1.0'.",
        file=sys.stderr,
    )
    sys.exit(1)

end_index = len(lines)
for index in range(target_index + 1, len(lines)):
    if heading.match(lines[index].strip()):
        end_index = index
        break

section = "\n".join(lines[target_index + 1:end_index]).strip()
if not section:
    print(
        f"CHANGELOG.md section for version {version} is empty.",
        file=sys.stderr,
    )
    sys.exit(1)

print(section)
PY
