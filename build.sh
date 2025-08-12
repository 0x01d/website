#!/bin/bash
set -e

echo "🦀 Setting up Rust environment..."

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
    source $HOME/.cargo/env
else
    echo "Rust already installed"
fi

echo "Rustup default stable"
rustup default stable

# Add wasm target
echo "Adding WASM target..."
rustup target add wasm32-unknown-unknown

# Install trunk if not present
if ! command -v trunk &> /dev/null; then
    echo "Installing trunk..."
    cargo install trunk
else
    echo "Trunk already installed"
fi

# Optional: Install wasm-opt for smaller binaries
# cargo install wasm-opt
cd ratzilla_app
echo "📦 Building WASM application..."
trunk build --release

echo "✅ Build complete!"
