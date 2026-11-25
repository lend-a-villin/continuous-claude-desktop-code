#!/bin/sh
# Builds CCDC.app (universal: Apple Silicon + Intel) and a .dmg. Run on macOS.
# Not signed with a Developer ID: the app is only ad-hoc signed, so macOS asks
# the user to allow it once (see the README).
set -eu
cd "$(dirname "$0")/../.."
VERSION=$(sed -n 's/^macos = "\(.*\)"/\1/p' Cargo.toml | head -1)  # [package.metadata.ccdc]

rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

OUT=target/macos
APP="$OUT/CCDC.app"
rm -rf "$OUT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/ccdc" \
  target/aarch64-apple-darwin/release/ccdc target/x86_64-apple-darwin/release/ccdc
sed "s/VERSION/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"

# icon: the PNGs build.rs wrote, turned into an .icns
ICONSET="$OUT/ccdc.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  cp "target/icons/ccdc-$s.png" "$ICONSET/icon_${s}x${s}.png"
  d=$((s * 2))
  cp "target/icons/ccdc-$d.png" "$ICONSET/icon_${s}x${s}@2x.png"
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/ccdc.icns"
cp LICENSE "$APP/Contents/Resources/LICENSE"

codesign --force --deep --sign - "$APP"

STAGE="$OUT/dmg"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
mkdir -p dist
hdiutil create -volname "CCDC $VERSION" -srcfolder "$STAGE" -ov -format UDZO "dist/CCDC-$VERSION-macos-universal.dmg"
echo "dist/CCDC-$VERSION-macos-universal.dmg"
