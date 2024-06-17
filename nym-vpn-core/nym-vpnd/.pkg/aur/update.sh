#! /bin/bash

# update PKGBUILD

set -E
set -o pipefail
# catch errors
trap 'catch $? ${FUNCNAME[0]:-main} $LINENO' ERR

catch() {
  >&2 echo " ✗ unexpected error, [$1] $2 L#$3"
  exit 1
}

if [ -z "$PKGBUILD" ]; then
  >&2 echo " ✕ PKGBUILD not set"
  exit 1
fi

if [ -z "$PKGNAME" ]; then
  >&2 echo " ✕ PKGNAME not set"
  exit 1
fi

if [ -z "$PKGVER" ]; then
  >&2 echo " ✕ PKGVER not set"
  exit 1
fi

if [ -z "$RELEASE_TAG" ]; then
  >&2 echo " ✕ RELEASE_TAG not set"
  exit 1
fi

if ! [ -a "$PKGBUILD" ]; then
  >&2 echo " ✕ no such file $PKGBUILD"
  exit 1
fi

if ! [ -a "$ARTIFACT" ]; then
  >&2 echo " ✕ no such file $ARTIFACT"
  exit 1
fi

# ⚠ pkgver does not allow dashes, replace any - by _
PKGVER=${PKGVER//-/_}

# bump package version
sed -i "s/pkgver=.*/pkgver=$PKGVER/" "$PKGBUILD"
echo " ✓ bump package version to $PKGVER"

if [ -n "$PKGREL" ]; then
  # ⚠ on new package version, pkgrel must be reset to 1
  sed -i "s/pkgrel=.*/pkgrel=$PKGREL/" "$PKGBUILD"
  echo " ✓ bump package rel to $PKGREL"
fi

# ⚠ order is important and should match the order of sources array
# declared in the PKGBUILD
sources=("$ARTIFACT" 'nym-vpnd.service')
sums=()

for file in "${sources[@]}"; do
  sum=$(sha256sum "$file" | awk '{print $1}')
  sums+=("\n    '$sum'")
  echo "sha256sum for $file: $sum"
done

sed -i "s/sha256sums=.*/sha256sums=(${sums[*]})/" "$PKGBUILD"
echo " ✓ updated checksums"

exit 0
