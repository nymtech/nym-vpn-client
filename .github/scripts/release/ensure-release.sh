#!/usr/bin/env bash
# Ensure the unified release object exists in the right state.
# Inputs (env): MODE (ship|nightly), TAG, TARGET_SHA, NOTES (optional)
set -euo pipefail

MODE="${MODE:?MODE required}"
TAG="${TAG:?TAG required}"
TARGET_SHA="${TARGET_SHA:?TARGET_SHA required}"
NOTES="${NOTES:-Assembled by release-all-platforms.}"

if [ "$MODE" = "ship" ]; then
  if gh release view "$TAG" >/dev/null 2>&1; then
    isDraft="$(gh release view "$TAG" --json isDraft -q .isDraft)"
    if [ "$isDraft" != "true" ]; then
      echo "::error:: ${TAG} already published — cannot amend an immutable release"
      exit 1
    fi
    echo "::notice:: reusing existing draft ${TAG}"
  else
    gh release create "$TAG" --draft --title "$TAG" --notes "$NOTES" --target "$TARGET_SHA"
    echo "::notice:: created draft ${TAG}"
  fi
else
  # Nightly: delete every prior nightly release + tag. Native immutable releases
  # ALLOW deleting a published release and its tag — they only forbid REUSING the
  # tag name, which we never do (TAG is freshly timestamped each run). Then create
  # a fresh DRAFT; the orchestrator's finalize-nightly job publishes it once all
  # assets are attached.
  while IFS= read -r t; do
    [ -n "$t" ] || continue
    echo "::notice:: deleting prior nightly ${t}"
    gh release delete "$t" --yes --cleanup-tag || true
  done < <(gh release list --json tagName -q '.[] | select(.tagName | startswith("nym-vpn-nightly")) | .tagName')

  gh release create "$TAG" --draft --prerelease --title "$TAG" --notes "$NOTES" --target "$TARGET_SHA"
  echo "::notice:: created nightly draft ${TAG}"
fi
