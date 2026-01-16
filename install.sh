#!/bin/bash
set -e

REPO_URL="https://github.com/lynn1286/feishu-bot.git"
TEMP_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "🚀 Installing Feishu Bot..."

# Check dependencies
command -v git >/dev/null 2>&1 || { echo "❌ git is required but not installed."; exit 1; }
command -v pnpm >/dev/null 2>&1 || { echo "❌ pnpm is required but not installed."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ cargo is required but not installed."; exit 1; }

# Clone repository
echo "📥 Cloning repository..."
git clone --depth 1 "$REPO_URL" "$TEMP_DIR"
cd "$TEMP_DIR"

# Build frontend
echo "📦 Building frontend..."
cd web
pnpm install
pnpm run build
cd ..

# Build backend
echo "🔧 Building backend..."
cargo build --release

# Install binary
echo "📥 Installing feishu-bot to ~/.cargo/bin..."
cargo install --path .

echo ""
echo "✅ Installation complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📌 首次启动会自动创建 admin 用户"
echo "   随机密码会显示在终端中，请登录后尽快修改"
echo ""
echo "💡 Usage:"
echo ""
echo "  feishu-bot serve      # Start server (foreground)"
echo "  feishu-bot admin      # Open admin panel in browser"
echo "  feishu-bot update     # Update to latest version"
echo "  feishu-bot uninstall  # Uninstall feishu-bot"
echo ""
