#!/bin/sh
# Runs on a GitHub macOS machine after make-dmg.sh: checks the built CCDC.app
# on a sample Claude data folder, then starts the menu bar app for a few seconds.
set -eux
APP=target/macos/CCDC.app
BIN="$APP/Contents/MacOS/ccdc"
"$BIN" version
lipo -archs "$BIN"
codesign --verify --verbose "$APP"

# a sample Claude data folder: two accounts, one session
D=$(mktemp -d)/Claude
A=11111111-1111-4111-8111-111111111111
B=22222222-2222-4222-8222-222222222222
O=33333333-3333-4333-8333-333333333333
S=44444444-4444-4444-8444-444444444444
mkdir -p "$D/claude-code-sessions/$A/$O" "$D/claude-code-sessions/$B/$O" "$D/logs"
echo "{\"lastActivityAt\": 1700000000000, \"title\": \"test\"}" > "$D/claude-code-sessions/$A/$O/local_$S.json"
echo "[info] [LocalSessionManager] Initialization succeeded — accountId=$A, orgId=$O" > "$D/logs/main.log"
export CCDC_DATA_DIR="$D"
export CCDC_APP_LOG="$D/logs/main.log"
"$BIN" status
"$BIN" replicate
test -f "$D/claude-code-sessions/$B/$O/local_$S.json"
# the copy keeps the source's modified time, so nothing is left to copy
"$BIN" replicate --dry-run | grep "Would write 0 file"
"$BIN" move --dry-run

# the menu bar app starts and keeps running
"$BIN" &
PID=$!
sleep 8
kill -0 "$PID"
kill "$PID"
cat "$HOME/Library/Application Support/CCDC/watch.log"
# first start turned on "start with the computer"
test -f "$HOME/Library/LaunchAgents/io.github.lend-a-villin.ccdc.plist"
cat "$HOME/Library/LaunchAgents/io.github.lend-a-villin.ccdc.plist"
