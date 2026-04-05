#!/usr/bin/env bash
# Updates the claude-rules submodule in all downstream repos.
# Run from anywhere. Optionally pass --push to also push each repo.

set -uo pipefail

PROJECTS_DIR="${PROJECTS_DIR:-$HOME/projects}"
PUSH=false

if [[ "${1:-}" == "--push" ]]; then
  PUSH=true
fi

REPOS=(
  protobuf-rs
  flatbuffers-rs
  codegen-infra
  fbsviewer
  fbsviewer-lib
  protoviewer-lib
  arrow-adbc-rs
  prisma-rs
  pure-grpc-rs
  grpcurl-rs
  issue-tracker-lite
  pwright
  myfeed
  beu
  rterm
  open-plx
  litevikings
  ast-cli
  narutoboardgame
  narutogame
  myissuetracker
  heydb
  dragb
  openworkspace
)

updated=0
skipped=0
failed=0

for repo in "${REPOS[@]}"; do
  dir="$PROJECTS_DIR/$repo"
  submodule_path=".claude/rules/shared"

  if [[ ! -d "$dir/$submodule_path" ]]; then
    echo "SKIP  $repo (no submodule at $submodule_path)"
    ((skipped++))
    continue
  fi

  echo -n "UPDATE $repo ... "

  if ! (cd "$dir" && git submodule update --remote "$submodule_path" 2>/dev/null); then
    echo "FAILED (submodule update)"
    ((failed++))
    continue
  fi

  if (cd "$dir" && git diff --quiet "$submodule_path"); then
    echo "already up to date"
    ((skipped++))
    continue
  fi

  (cd "$dir" && git add "$submodule_path" && git commit -m "Update shared claude-rules" --quiet)
  echo "committed"

  if $PUSH; then
    if (cd "$dir" && git push --quiet 2>/dev/null); then
      echo "       pushed"
    else
      echo "       push FAILED"
      ((failed++))
      continue
    fi
  fi

  ((updated++))
done

echo ""
echo "Done: $updated updated, $skipped skipped, $failed failed"
