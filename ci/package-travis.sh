# ci/package-travis.sh
#!/usr/bin/env bash
set -euo pipefail

CRATE_NAME="$1"
TARGET="$2"
OSNAME="$3"

# where cargo built the binary:
BIN="target/${TARGET}/release/${CRATE_NAME}"

# Output dir
mkdir -p release

# Compute human-friendly asset name:
# linux => pet-linux-x86_64, macOS => pet-macos-x86_64
case "$TARGET" in
  x86_64-unknown-linux-gnu)   OUT="${CRATE_NAME}-linux-x86_64" ;;
  x86_64-apple-darwin)        OUT="${CRATE_NAME}-macos-x86_64" ;;
  aarch64-apple-darwin)       OUT="${CRATE_NAME}-macos-arm64" ;;
  *) echo "Unknown target $TARGET"; exit 1 ;;
esac

# Package (tar.gz) and checksum
STAGE="$(mktemp -d)"
cp "$BIN" "$STAGE/$CRATE_NAME"
chmod +x "$STAGE/$CRATE_NAME"

tar -C "$STAGE" -czf "release/${OUT}.tar.gz" "$CRATE_NAME"
( cd release && shasum -a 256 "${OUT}.tar.gz" > "${OUT}.tar.gz.sha256" )

echo "Packaged release/${OUT}.tar.gz"
