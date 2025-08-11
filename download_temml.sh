#!/usr/bin/env bash

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
VENDOR_DIR="$ROOT_DIR/vendor/temml"

VERSION="$(cat ./TEMML-VERSION)"
URL="https://github.com/ronkok/Temml/archive/refs/tags/v${VERSION}.tar.gz"

rm -rf "$VENDOR_DIR"
mkdir -p "$VENDOR_DIR"
echo "download ${URL}..."
curl -L https://raw.githubusercontent.com/ronkok/Temml/master/LICENSE -o "$VENDOR_DIR/TEMML-LICENSE"
BASENAME="Temml-${VERSION}"
curl -L "$URL" | tar -x -z -C "$VENDOR_DIR" --strip-components 1 -f - "${BASENAME}/dist/temml.min.js" "${BASENAME}/contrib/mhchem/mhchem.min.js" "${BASENAME}/contrib/physics/physics.js" "${BASENAME}/contrib/texvc/texvc.js"

