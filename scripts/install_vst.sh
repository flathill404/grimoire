#!/bin/bash
set -e

# ==============================================================================
# VST3 Plugin Installer Script
#
# Description:
#   Links the bundled VST3 plugin from the target directory to the user's
#   VST3 directory and removes MacOS quarantine attributes.
#
# Usage:
#   ./scripts/install_vst.sh <plugin_name>
# ==============================================================================

# 1. Validate arguments
if [ -z "$1" ]; then
    echo "Error: No plugin name provided."
    echo "Usage: $0 <plugin_name>"
    echo "Example: $0 cantrip_gain"
    exit 1
fi

PLUGIN_NAME="$1"

# 2. Resolve paths
# Get the absolute path of the directory where this script resides (e.g., /.../grimoire/scripts)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# The project root is one level up from the scripts directory
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VST3_SOURCE_DIR="$PROJECT_ROOT/target/bundled"
VST3_SOURCE="$VST3_SOURCE_DIR/$PLUGIN_NAME.vst3"
VST3_DEST_DIR="$HOME/Library/Audio/Plug-Ins/VST3"
VST3_DEST="$VST3_DEST_DIR/$PLUGIN_NAME.vst3"

# 3. Check if the build artifact exists
if [ ! -d "$VST3_SOURCE" ]; then
    echo "Error: VST3 bundle not found at: $VST3_SOURCE"
    echo "Please build the plugin first using:"
    echo "  cargo xtask bundle $PLUGIN_NAME --release"
    exit 1
fi

# 4. Ensure destination directory exists
if [ ! -d "$VST3_DEST_DIR" ]; then
    echo "Creating VST3 directory at: $VST3_DEST_DIR"
    mkdir -p "$VST3_DEST_DIR"
fi

# 5. Remove existing plugin (file or symlink) to avoid conflicts
if [ -e "$VST3_DEST" ]; then
    echo "Removing existing plugin..."
    rm -rf "$VST3_DEST"
fi

# 6. Create symbolic link
echo "Installing $PLUGIN_NAME to $VST3_DEST_DIR ..."
ln -s "$VST3_SOURCE" "$VST3_DEST_DIR/"

# 7. Remove MacOS quarantine attributes (Gatekeeper)
# This prevents "app is damaged" or verification errors in DAWs
echo "Removing MacOS quarantine attributes..."
xattr -cr "$VST3_DEST"

echo "--------------------------------------------------"
echo "Success! $PLUGIN_NAME has been installed."
echo "You may need to rescan plugins in your DAW."
echo "--------------------------------------------------"