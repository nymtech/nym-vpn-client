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

tarball="$RELEASE_TAG.tar.gz"

if ! [ -a "$tarball" ]; then
  >&2 echo " ✕ no such file $tarball"
  exit 1
fi

# bump package version
sed -i "s/pkgver=.*/pkgver=$PKGVER/" "$PKGBUILD"
echo " ✓ bump package version to $PKGVER"

if [ -n "$PKGREL" ]; then
  sed -i "s/pkgrel=.*/pkgrel=$PKGREL/" "$PKGBUILD"
  echo " ✓ bump package rel to $PKGREL"
fi

sources=("$tarball" '.pkg/aur/nymvpn-x-wrapper.sh' '.pkg/aur/nymvpn-x.desktop' '.pkg/aur/nymvpn-x.svg')
sums=()

for file in "${sources[@]}"; do
  sum=$(sha256sum "$file" | awk '{print $1}')
  sums+=("\n    '$sum'")
  echo "sha256sum for $file: $sum"
done

sed -i "s/sha256sums=.*/sha256sums=(${sums[*]})/" "$PKGBUILD"
echo " ✓ updated checksums"

exit 0
