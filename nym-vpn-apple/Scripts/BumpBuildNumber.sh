#!/bin/bash

cd ..

# Check for any Git changes
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Error: Uncommitted changes found."
  exit 1
else
  git checkout develop
  git pull
  git branch -D feat/bumpBuild
  git checkout feat/bumpBuild
  fastlane mac bump_build
  git add .
  git commit -m "Apple: Bump build number"
  git push
fi
