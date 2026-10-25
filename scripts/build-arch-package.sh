#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
for tool in tar sha256sum sed makepkg cargo; do
  command -v "$tool" >/dev/null || { echo "Missing required command: $tool" >&2; exit 1; }
done
if [[ ${EUID:-$(id -u)} -eq 0 ]]; then echo 'Do not run makepkg as root.' >&2; exit 1; fi
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
BUILD="$PWD/packaging/arch/local-build"
mkdir -p "$BUILD"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT
mkdir -p "$STAGING/bandhop-${VERSION}"
tar --exclude='./target' --exclude='./.git' --exclude='./packaging/arch/local-build' \
  --exclude='./*.pkg.tar.*' -cf - . | tar -xf - -C "$STAGING/bandhop-${VERSION}"
tar -czf "$BUILD/bandhop-${VERSION}.tar.gz" -C "$STAGING" "bandhop-${VERSION}"
HASH="$(sha256sum "$BUILD/bandhop-${VERSION}.tar.gz" | cut -d' ' -f1)"
sed "s/@SHA256@/$HASH/" packaging/arch/PKGBUILD.local.in > "$BUILD/PKGBUILD"
echo "Building local Bandhop ${VERSION} using $BUILD/PKGBUILD"
(cd "$BUILD" && makepkg -s)
