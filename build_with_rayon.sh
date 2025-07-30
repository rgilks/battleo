#!/bin/bash

# Build script for BattleO with Rayon WASM support
# This script builds the project with wasm-bindgen-rayon enabled

set -e

echo "🚀 Building BattleO with Rayon WASM support..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Check if target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean
rm -rf pkg/

# Build with wasm-bindgen-rayon feature using nightly Rust
echo "🔨 Building with wasm-bindgen-rayon feature (nightly Rust)..."
RUSTUP_TOOLCHAIN=nightly wasm-pack build --target web --features wasm-bindgen-rayon

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build completed successfully!"
    echo ""
    echo "📁 Generated files in pkg/:"
    ls -la pkg/
    echo ""
    echo "🎯 To test Rayon WASM functionality:"
    echo "   1. Start a local server: python3 -m http.server 8000"
    echo "   2. Open http://localhost:8000/rayon_demo.html"
    echo "   3. Click 'Initialize Rayon Thread Pool'"
    echo "   4. Run performance tests"
    echo ""
    echo "🔧 Build options used:"
    echo "   - Target: web"
    echo "   - Features: wasm-bindgen-rayon"
    echo "   - Optimization: disabled (for debugging)"
else
    echo "❌ Build failed!"
    exit 1
fi 