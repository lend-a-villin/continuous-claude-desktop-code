#!/bin/sh
# Runs inside a clean Debian/Ubuntu/Mint container (see .github/workflows/build.yml):
# installs the .deb with its dependencies, then checks the command line and the tray.
set -eux
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y ./dist/*.deb xvfb dbus dbus-x11 >/dev/null
ccdc version

# a sample Claude data folder: two accounts, one session
D=/tmp/claude-data
A=11111111-1111-4111-8111-111111111111
B=22222222-2222-4222-8222-222222222222
O=33333333-3333-4333-8333-333333333333
S=44444444-4444-4444-8444-444444444444
mkdir -p "$D/claude-code-sessions/$A/$O" "$D/claude-code-sessions/$B/$O" "$D/logs"
echo "{\"lastActivityAt\": 1700000000000, \"title\": \"test\"}" > "$D/claude-code-sessions/$A/$O/local_$S.json"
echo "[info] [LocalSessionManager] Initialization succeeded — accountId=$A, orgId=$O" > "$D/logs/main.log"
export CCDC_DATA_DIR="$D"
ccdc status
ccdc replicate
test -f "$D/claude-code-sessions/$B/$O/local_$S.json"
# the copy keeps the source's modified time, so nothing is left to copy
ccdc replicate --dry-run | grep "Would write 0 file"
ccdc move --dry-run

# the tray starts and keeps running (killed after 8 s: exit code 124)
set +e
dbus-run-session -- xvfb-run -a timeout 8 ccdc
code=$?
set -e
test "$code" = 124
cat "$HOME/.config/ccdc/watch.log"
