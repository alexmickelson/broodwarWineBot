#!/usr/bin/env nix-shell
#! nix-shell -i bash -I nixpkgs=https://github.com/NixOS/nixpkgs/archive/nixos-24.11.tar.gz -p wineWowPackages.stable

wine --version

set -e

# Parse command line arguments
USE_RUST_BOT=false
if [ "$1" = "--rust" ]; then
    USE_RUST_BOT=true
    echo "Using Rust bot"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export WINEPREFIX="$SCRIPT_DIR/.wine"
export WINEARCH=win64
export DISPLAY=:0
export WINEDLLOVERRIDES="mscoree,mshtml="
export WINEDEBUG=-all

if [ ! -d "$WINEPREFIX" ]; then
    wine wineboot --init
fi

INSTALL_DIR="$SCRIPT_DIR/starcraft"
# Convert Unix path to Windows path for Wine registry
SC_WIN_PATH="$(winepath -w "$INSTALL_DIR" 2>/dev/null || echo "Z:${INSTALL_DIR}" | sed 's/\//\\\\/g')\\\\"


configure_registry() {
    # Core registry entries
    wine REG ADD "HKEY_LOCAL_MACHINE\\SOFTWARE\\Blizzard Entertainment\\Starcraft" \
        /v InstallPath /t REG_EXPAND_SZ /d "${SC_WIN_PATH}" /f 2>/dev/null || true

    wine REG ADD "HKEY_LOCAL_MACHINE\\SOFTWARE\\Blizzard Entertainment\\Starcraft" \
        /v Program /t REG_EXPAND_SZ /d "${SC_WIN_PATH}StarCraft.exe" /f 2>/dev/null || true

    # Disable Intro
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Blizzard Entertainment\\Starcraft" /v "Intro" /t REG_SZ /d "0" /f 2>/dev/null || true
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Blizzard Entertainment\\Starcraft" /v "IntroX" /t REG_SZ /d "0" /f 2>/dev/null || true
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Blizzard Entertainment\\Starcraft" /v "Tip" /t REG_SZ /d "0" /f 2>/dev/null || true

    # Chaoslauncher configuration
    SC_CHAOS_PATH=$(winepath -w "$INSTALL_DIR" | sed 's/\\/\\\\/g')
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\Launcher" /v "ScPath" /t REG_SZ /d "${SC_CHAOS_PATH}" /f 2>/dev/null || true
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\Launcher" /v "GameVersion" /t REG_SZ /d "Starcraft 1.16.1" /f 2>/dev/null || true
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\Launcher" /v "RunScOnStartup" /t REG_SZ /d "1" /f 2>/dev/null || true

    # Enable BWAPI 4.4.0 Injector plugin by default
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\PluginsEnabled" /v "BWAPI 4.4.0 Injector [RELEASE]" /t REG_SZ /d "1" /f 2>/dev/null || true
    wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\PluginsEnabled" /v "W-MODE 1.02" /t REG_SZ /d "1" /f 2>/dev/null || true
}



echo "Starting Xvfb virtual display..."
Xvfb :0 -auth ~/.Xauthority -screen 0 640x480x24 > /dev/null 2>&1 &
XVFB_PID=$!

configure_registry

if [ "$USE_RUST_BOT" = true ]; then
    echo "Starting RustBot..."
    cd "$SCRIPT_DIR/rustbot/target/x86_64-pc-windows-gnu/debug"
    wine rustbot.exe &
    BOT_PID=$!
    echo "RustBot started (PID: $BOT_PID)"
else
    echo "Starting StarterBot..."
    cd "$SCRIPT_DIR/bin_linux"
    wine StarterBot.exe &
    BOT_PID=$!
    echo "StarterBot started (PID: $BOT_PID)"
fi

echo "Launching StarCraft with BWAPI via Chaoslauncher..."
cd "$SCRIPT_DIR/starcraft/BWAPI/Chaoslauncher"
wine Chaoslauncher.exe

echo "StarCraft closed. Cleaning up..."
if kill -0 $XVFB_PID 2>/dev/null; then
    echo "Stopping Xvfb..."
    kill $XVFB_PID 2>/dev/null || true
fi

if kill -0 $BOT_PID 2>/dev/null; then
    echo "Stopping StarterBot..."
    kill $BOT_PID 2>/dev/null || true
fi
echo "Done!"
