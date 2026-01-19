#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_PATH="${SCRIPT_DIR}/scripts"

export WINEPREFIX="$SCRIPT_DIR/.wine"
export WINEARCH=win64
export DISPLAY=:0
export WINEDLLOVERRIDES="mscoree,mshtml="
export WINEDEBUG=-all

# Cleanup function to ensure processes are killed on exit
cleanup() {
    echo ""
    echo "Cleaning up processes..."
    if [ -n "$XVFB_PID" ] && kill -0 $XVFB_PID 2>/dev/null; then
        echo "Stopping Xvfb..."
        kill $XVFB_PID 2>/dev/null || true
    fi
    if [ -n "$BOT_PID" ] && kill -0 $BOT_PID 2>/dev/null; then
        echo "Stopping RustBot..."
        kill $BOT_PID 2>/dev/null || true
    fi
    killall StarCraft.exe
    echo "Cleanup complete."
}

# Register cleanup function to run on script exit (success or failure)
trap cleanup EXIT

if [ ! -d "$WINEPREFIX" ]; then
    wine wineboot --init
fi

echo "Starting Xvfb virtual display..."
Xvfb :0 -auth ~/.Xauthority -screen 0 640x480x24 > /dev/null 2>&1 &
XVFB_PID=$!

cd scripts
    ./4-configure-bwapi.sh
cd ..

echo "Building RustBot..."
# build-rustbot-debug
nix develop -c build-rustbot-debug
echo "Starting RustBot..."
cd "$SCRIPT_DIR/rustbot"

RUST_BACKTRACE=1 RUST_BACKTRACE=full wine target/x86_64-pc-windows-gnu/debug/rustbot.exe &
BOT_PID=$!
echo "RustBot started (PID: $BOT_PID)"


echo "Launching StarCraft with BWAPI via Chaoslauncher..."
cd "$SCRIPT_DIR/starcraft/BWAPI/Chaoslauncher"
wine Chaoslauncher.exe

echo "StarCraft closed."
