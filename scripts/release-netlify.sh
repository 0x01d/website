#!/usr/bin/env bash
set -euo pipefail

export BUILD_SKIP_HOOK=true
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
WORKTREE_DIR=".deploy_netlify"

cleanup() {
    echo "🧹 Cleaning up worktree..."
    git worktree remove "$WORKTREE_DIR" --force 2>/dev/null || true
}
trap cleanup EXIT INT ERR

# Ensure netlify.toml exists
if [ ! -f "netlify.toml" ]; then
    echo "❌ netlify.toml is missing. Aborting deploy."
    exit 1
fi

echo "🌐 Building production files with trunk..."
trunk build --release --dist dist-prod

echo "🔄 Preparing worktree for release-netlify..."
# If branch exists, add it. If not, create it.
if git show-ref --verify --quiet refs/heads/release-netlify; then
    git worktree add "$WORKTREE_DIR" release-netlify
else
    git worktree add -b release-netlify "$WORKTREE_DIR"
fi

echo "✅ Syncing dist-prod and netlify.toml into worktree..."
rsync -av --delete dist-prod/ "$WORKTREE_DIR/dist-prod/"
cp netlify.toml "$WORKTREE_DIR/"

echo "📦 Committing and pushing from worktree..."
(
    cd "$WORKTREE_DIR"
    git add dist-prod netlify.toml
    git commit -m "Deploy: update dist-prod for Netlify" || echo "Nothing to commit."
    git push origin release-netlify
)

echo "✅ Done! Your site is live on Netlify."
