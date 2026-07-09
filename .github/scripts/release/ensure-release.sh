#!/usr/bin/env bash
# Ensure the unified release object exists in the right state for the channel.
# Inputs (env): CHANNEL (nightly|beta|ship), TAG, TARGET_SHA, NOTES (optional)
#
#   nightly : delete every prior nightly release+tag, create a fresh DRAFT --prerelease.
#             finalize-nightly publishes it once assets are attached.
#   beta    : create-or-reuse a DRAFT --prerelease (human publishes). Prior betas kept.
#   ship    : create-or-reuse a DRAFT (human publishes). Fail if already published.
set -euo pipefail

CHANNEL="${CHANNEL:?CHANNEL required (nightly|beta|ship)}"
TAG="${TAG:?TAG required}"
TARGET_SHA="${TARGET_SHA:?TARGET_SHA required}"
NOTES="${NOTES:-Assembled by release-all-platforms.}"

case "$CHANNEL" in
  nightly)
    # Native immutable releases ALLOW deleting a published release+tag (they only forbid
    # REUSING a tag name — the date-only nightly tag never reuses across days).
    # Match ONLY unified nightly tags: legacy "nym-vpn-nightly-<ts>" and the new
    # "nym-vpn-v<base>-nightly.<date>". Deliberately NOT the standalone per-platform
    # nightlies (macos-nightly, nym-vpn-app-nightly, nym-vpn-core-nightly).
    while IFS= read -r t; do
      [ -n "$t" ] || continue
      echo "::notice:: deleting prior nightly ${t}"
      gh release delete "$t" --yes --cleanup-tag || true
    done < <(gh release list --json tagName -q '.[] | select(.tagName | test("^nym-vpn-(nightly-|v.*-nightly[.])")) | .tagName')

    gh release create "$TAG" --draft --prerelease --title "$TAG" --notes "$NOTES" --target "$TARGET_SHA"
    echo "::notice:: created nightly draft ${TAG}"
    ;;

  beta|ship)
    EXTRA=()
    [ "$CHANNEL" = "beta" ] && EXTRA+=(--prerelease)

    if gh release view "$TAG" >/dev/null 2>&1; then
      isDraft="$(gh release view "$TAG" --json isDraft -q .isDraft)"
      if [ "$isDraft" != "true" ]; then
        echo "::error:: ${TAG} already published — cannot amend an immutable release"
        exit 1
      fi
      echo "::notice:: reusing existing draft ${TAG}"
    else
      gh release create "$TAG" --draft "${EXTRA[@]}" --title "$TAG" --notes "$NOTES" --target "$TARGET_SHA"
      echo "::notice:: created ${CHANNEL} draft ${TAG}"
    fi
    ;;

  *)
    echo "::error:: unknown release channel: ${CHANNEL} (expected nightly|beta|ship)"
    exit 1
    ;;
esac
