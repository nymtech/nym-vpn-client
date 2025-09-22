#!/bin/bash
set -euo pipefail

# Fetch remote refs
git fetch origin --quiet

current_branch=$(git rev-parse --abbrev-ref HEAD)
echo "🔍 Current branch: $current_branch"

# Candidate base branches
candidates=(main master develop)
# Add all remote release/* branches dynamically
release_branches=$(git branch -r --list "origin/release/*" | sed 's|origin/||')
candidates+=($release_branches)

best_base=""
best_distance=1000

for candidate in "${candidates[@]}"; do
  if git show-ref --verify --quiet "refs/remotes/origin/$candidate"; then
    merge_base=$(git merge-base "$current_branch" "origin/$candidate")
    if [[ -n "$merge_base" ]]; then
      # Distance = number of commits since branching
      distance=$(git rev-list --count "$merge_base..$current_branch")
      if (( distance < best_distance )); then
        best_distance=$distance
        best_base=$candidate
      fi
    fi
  fi
done

if [[ -n "$best_base" ]]; then
  echo "✅ Likely base branch: $best_base (diverged $best_distance commits ago)"
  sh FetchCore.sh "$best_base"
else
  echo "❌ Could not determine base branch."
fi
