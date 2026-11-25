#!/bin/sh
# Builds dist/CCDC-<version>-linux-amd64.deb. Run on Ubuntu 22.04 (the oldest
# supported system, so the package also runs on newer Ubuntu, Debian 12+ and
# Linux Mint 21+). In a bare container (run as root) it first installs the
# build tools; on a CI machine they are already there.
#   docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
set -eu
cd "$(dirname "$0")/../.."
export DEBIAN_FRONTEND=noninteractive
SUDO=""; [ "$(id -u)" = 0 ] || SUDO="sudo"
$SUDO apt-get update -q
$SUDO apt-get install -y -q build-essential pkg-config curl ca-certificates \
  libgtk-3-dev libayatana-appindicator3-dev libxdo-dev >/dev/null
if ! command -v cargo >/dev/null; then
  [ -x "$HOME/.cargo/bin/cargo" ] || curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  export PATH="$HOME/.cargo/bin:$PATH"
fi
command -v cargo-deb >/dev/null || cargo install cargo-deb --locked

VERSION=$(sed -n 's/^linux = "\(.*\)"/\1/p' Cargo.toml | head -1)  # [package.metadata.ccdc]
cargo test --release
cargo deb --deb-version "$VERSION"
mkdir -p dist
cp "${CARGO_TARGET_DIR:-target}/debian/ccdc_${VERSION}_amd64.deb" "dist/CCDC-$VERSION-linux-amd64.deb"
echo "dist/CCDC-$VERSION-linux-amd64.deb"
