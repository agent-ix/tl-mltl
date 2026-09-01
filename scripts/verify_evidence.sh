#!/usr/bin/env bash
set -euo pipefail

found=0
active=0
if [[ -n "$(/usr/bin/git status --porcelain --untracked-files=all)" ]]; then
  echo "evidence verification requires a clean source tree" >&2
  exit 1
fi
if [[ ! -f evidence/ANCHORS ]]; then
  echo "retained evidence anchor manifest is missing" >&2
  exit 1
fi
/usr/bin/sha256sum --check evidence/ANCHORS
while IFS= read -r -d '' checksum; do
  found=1
  if ! /usr/bin/grep -Fqx "$(/usr/bin/sha256sum "$checksum")" evidence/ANCHORS; then
    echo "retained evidence manifest lacks a committed anchor: $checksum" >&2
    exit 1
  fi
  /usr/bin/sha256sum --check "$checksum"
  evidence_dir="${checksum%.sha256}"
  profile="$(/usr/bin/python3 scripts/evidence_profile.py "$evidence_dir")"
  if [[ "$profile" == retracted ]]; then
    continue
  fi
  if [[ "$profile" != v2 ]]; then
    echo "retained evidence is not an active reachable qualification-v2 record: $evidence_dir" >&2
    exit 1
  fi
  active=1
  /usr/bin/python3 scripts/finalize_collection.py --check "$evidence_dir"
done < <(/usr/bin/find evidence -maxdepth 1 -type f -name '*.sha256' -print0 | /usr/bin/sort -z)

if [[ $found -eq 0 ]]; then
  echo "no retained evidence checksum manifests found" >&2
  exit 1
fi
if [[ $active -eq 0 ]]; then
  echo "no active non-retracted evidence record supports qualification" >&2
  exit 1
fi
