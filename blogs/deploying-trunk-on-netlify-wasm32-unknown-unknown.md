# How I deploy kll.re, a ratzilla static site on netlify

This site is a wasm32-unknown-unkown binary that I deploy via `trunk-rs` to 
netlify. Since they don't support building a project with rust, I prebuild on my
machine and push it to a dedicated `netlify-release` branch containing only the 
`dist-prod` directory and the `netlify.toml`. 

To accomplish this I am using a little shell script that does a sparse checkout.
> Note: sparse checkout is in an experimental state, I used this because I wanted
> to keep the netlify branch pure and clean.

File: `scripts/netlify-release.sh`
```bash
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
``` 

Then I ran a quick command to add it as an alias in git.

```bash
git config alias.netlify-release '!sh scripts/netlify-release.sh'
```

Now I can easily run the script with:
```bash
git netlify-release
```

To stop my retarted ass from commiting all files to the `netlify-release` branch
and having to clumse around in the repo. I have made a pre-commit hook to enforce 
the use of the script locally.

File: `.git/hooks/pre-commit`
```bash
#!/bin/sh

BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Only apply to release-netlify branch
if [ "$BRANCH" = "release-netlify" ]; then
    # Allow if BUILD_SKIP_HOOK is set
    if [ "$BUILD_SKIP_HOOK" != "true" ]; then
        echo "❌ Direct commits to release-netlify are forbidden."
        echo "✅ Use: ./scripts/netlify-release.sh"
        exit 1
    fi
fi

exit 0
``` 

