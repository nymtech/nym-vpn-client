#! /bin/bash

# update PKGBUILD

set -E
set -o pipefail
# catch errors
trap 'catch $? ${FUNCNAME[0]:-main} $LINENO' ERR

catch() {
  echo >&2 " ✗ unexpected error, [$1] $2 L#$3"
  exit 1
}

if [ -z "$PKGBUILD" ]; then
  echo >&2 " ✕ PKGBUILD not set"
  exit 1
fi

if [ -z "$PKGNAME" ]; then
  echo >&2 " ✕ PKGNAME not set"
  exit 1
fi

if [ -z "$PKGVER" ]; then
  echo >&2 " ✕ PKGVER not set"
  exit 1
fi

if [ -z "$RELEASE_TAG" ]; then
  echo >&2 " ✕ RELEASE_TAG not set"
  exit 1
fi

if ! [ -a "$PKGBUILD" ]; then
  echo >&2 " ✕ no such file $PKGBUILD"
  exit 1
fi

if [ -z "$SOURCES" ]; then
  echo >&2 " ✕ SOURCES not set"
  exit 1
fi

# bump _pkgver
sed -i "s/_pkgver=.*/_pkgver=$PKGVER/" "$PKGBUILD"
echo " ✓ bump _pkgver to $PKGVER"

# ⚠ Dashes are not allowed in package version, replace any - by _
PKGVER=${PKGVER//-/_}

# bump package version
sed -i "s/pkgver=.*/pkgver=$PKGVER/" "$PKGBUILD"
echo " ✓ bump package version to $PKGVER"

# bump _release_tag
sed -i "s/_release_tag=.*/_release_tag=$RELEASE_TAG/" "$PKGBUILD"
echo " ✓ bump _release_tag to $RELEASE_TAG"

if [ -n "$PKGREL" ]; then
  # ⚠ on new package version, pkgrel must be reset to 1
  sed -i "s/pkgrel=.*/pkgrel=$PKGREL/" "$PKGBUILD"
  echo " ✓ bump package rel to $PKGREL"
fi

# SOURCES must be a string array with newline separated values
mapfile -t sources <<<"$SOURCES"
echo "sources: ${sources[*]}"
sums=()

for file in "${sources[@]}"; do
  if [ -n "$file" ]; then
    sum=$(sha256sum "$file" | awk '{print $1}')
    sums+=("\n    '$sum'")
    echo "sha256sum for $file: $sum"
  fi
done

sed -i "s/sha256sums=.*/sha256sums=(${sums[*]})/" "$PKGBUILD"
echo " ✓ updated checksums"

exit 0
