#!/bin/bash
set -e

export PATH="$HOME/.cargo/bin:$PATH"

echo "========================================"
echo "  StudioLink Full Build & Install"
echo "========================================"
echo ""

# 1. Rust server build
echo "[1/4] Building Rust server..."
cargo build --release 2>&1
echo "  -> Server binary: target/release/studiolink"
echo ""

# 2. Clippy check
echo "[2/4] Running clippy..."
cargo clippy 2>&1
echo "  -> Clippy passed"
echo ""

# 3. Build Roblox plugin with Rojo
echo "[3/4] Building Roblox plugin..."
cd plugin
rojo build -o StudioLink.rbxm 2>&1
cd ..
echo "  -> Plugin built: plugin/StudioLink.rbxm"
echo ""

# 4. Install plugin to Studio
PLUGIN_DIR="$HOME/Documents/Roblox/Plugins"
echo "[4/4] Installing plugin to Studio..."
mkdir -p "$PLUGIN_DIR"
cp plugin/StudioLink.rbxm "$PLUGIN_DIR/StudioLink.rbxm"
echo "  -> Installed to: $PLUGIN_DIR/StudioLink.rbxm"
echo ""

echo "========================================"
echo "  BUILD COMPLETE"
echo "========================================"
echo ""
echo "Next steps:"
echo "  1. Restart Roblox Studio (or reload plugins)"
echo "  2. Make sure HTTP Requests is enabled in Game Settings > Security"
echo "  3. Start the server: ./target/release/studiolink"
echo "     Or use it as MCP server in Claude/Cursor config"
echo ""
