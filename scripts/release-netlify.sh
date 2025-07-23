#!/usr/bin/env bash
set -e

export BUILD_SKIP_HOOK=true # Signal to pre-commit hook

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Ensure netlify.toml exists
if [ ! -f "netlify.toml" ]; then
    echo "❌ netlify.toml is missing. Aborting deploy."
    exit 1
fi

echo "🌐 Building production files with trunk..."
trunk build --release --dist dist-prod

echo "🔄 Switching to release-netlify branch..."
git stash push -m "temp-stash" || true
git checkout release-netlify || git checkout -b release-netlify

echo "🔍 Enabling sparse-checkout for dist-prod and netlify.toml..."
if ! git config core.sparseCheckout >/dev/null; then
    git sparse-checkout init --cone
fi
git sparse-checkout set dist-prod netlify.toml

echo "✅ Adding and committing changes..."
git add dist-prod netlify.toml
git commit -m "Deploy: update dist-prod for Netlify" || echo "Nothing to commit."

echo "🚀 Pushing to origin/release-netlify..."
git push origin release-netlify

echo "↩️ Switching back to $CURRENT_BRANCH..."
git checkout "$CURRENT_BRANCH"
git sparse-checkout disable
git stash pop || true

echo "✅ Done! Your site is live on Netlify."
