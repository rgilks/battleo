#!/bin/bash

# Build script for BattleO main application with Rayon WASM support
# This script builds the main application with wasm-bindgen-rayon enabled

set -e

echo "🚀 Building BattleO main application with Rayon WASM support..."

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
echo "🔨 Building main application with wasm-bindgen-rayon feature (nightly Rust)..."
RUSTUP_TOOLCHAIN=nightly wasm-pack build --target web --features wasm-bindgen-rayon

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build completed successfully!"
    echo ""
    echo "📁 Generated files in pkg/:"
    ls -la pkg/
    echo ""
    echo "🎯 To run the main application with Rayon support:"
    echo "   1. Start a local server: ./serve_demo.sh"
    echo "   2. Open http://localhost:8000 in your browser"
    echo "   3. Click 'Initialize Rayon' to enable parallel processing"
    echo "   4. Start the simulation and enjoy improved performance!"
    echo ""
    echo "🔧 Build options used:"
    echo "   - Target: web"
    echo "   - Features: wasm-bindgen-rayon"
    echo "   - Optimization: disabled (for debugging)"
    echo ""
    echo "📊 Performance features:"
    echo "   - Parallel agent updates"
    echo "   - Parallel resource processing"
    echo "   - Parallel statistics calculation"
    echo "   - Automatic fallback to sequential processing"
else
    echo "❌ Build failed!"
    exit 1
fi 