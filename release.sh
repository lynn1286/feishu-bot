#!/bin/bash
set -e

# 获取当前版本
CURRENT_VERSION=$(node -p "require('./package.json').version")
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

echo "🚀 Releasing Feishu Bot..."
echo "📌 Current version: $CURRENT_VERSION"
echo ""

# 检查是否有未提交的更改
if [[ -n $(git status --porcelain) ]]; then
  echo "❌ Error: Working directory is not clean. Please commit or stash changes first."
  exit 1
fi

# 版本选择
echo "Select release type:"
PS3="Choose (1-4): "
options=(
  "patch  ($CURRENT_VERSION -> $MAJOR.$MINOR.$((PATCH + 1)))"
  "minor ($CURRENT_VERSION -> $MAJOR.$((MINOR + 1)).0)"
  "major ($CURRENT_VERSION -> $((MAJOR + 1)).0.0)"
  "custom"
)

select opt in "${options[@]}"; do
  case $REPLY in
    1) TYPE="patch"; NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"; break ;;
    2) TYPE="minor"; NEW_VERSION="$MAJOR.$((MINOR + 1)).0"; break ;;
    3) TYPE="major"; NEW_VERSION="$((MAJOR + 1)).0.0"; break ;;
    4)
      read -p "Enter new version (e.g. 1.2.3): " NEW_VERSION
      if [[ ! $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "❌ Error: Invalid version format"
        exit 1
      fi
      break
      ;;
    *) echo "Invalid option. Try again." ;;
  esac
done

TAG_NAME="v$NEW_VERSION"
echo ""
echo "📦 New version: $NEW_VERSION"
echo "🏷️  Tag: $TAG_NAME"

# 确认
read -p "Continue? (y/N) " CONFIRM
if [[ ! $CONFIRM =~ ^[Yy]$ ]]; then
  echo "❌ Cancelled"
  exit 1
fi

# 更新 package.json 版本
echo ""
echo "📝 Updating package.json..."
npm version "$NEW_VERSION" --no-git-tag-version > /dev/null

# 提交版本更改
echo "📝 Committing version bump..."
git add package.json
git commit -m "chore(release): $NEW_VERSION"

# 创建 tag
echo "🏷️  Creating tag $TAG_NAME..."
git tag -a "$TAG_NAME" -m "Release $TAG_NAME"

# 推送
echo ""
echo "📤 Pushing to GitHub..."
git push origin main
git push origin "$TAG_NAME"

echo ""
echo "✅ Release complete!"
echo "   https://github.com/lynn1286/feishu-bot/releases/tag/$TAG_NAME"
