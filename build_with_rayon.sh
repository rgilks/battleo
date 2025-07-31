#!/bin/bash

# Build script for BattleO with Rayon WASM support
# This script builds the project with wasm-bindgen-rayon enabled and automatically fixes worker import issues

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
    
    # Fix Rayon worker import paths automatically
    echo "🔧 Fixing Rayon worker import paths..."
    
    # Find the workerHelpers.js file
    WORKER_FILE=$(find pkg/snippets -name "workerHelpers.js" -type f 2>/dev/null | head -n 1)
    
    if [ -z "$WORKER_FILE" ]; then
        echo "❌ No workerHelpers.js file found. Rayon workers may not work correctly."
    else
        echo "📁 Found worker file: $WORKER_FILE"
        
        # Create backup
        cp "$WORKER_FILE" "${WORKER_FILE}.backup"
        
        # Fix the import path to use absolute path
        echo "🔨 Fixing import path..."
        sed -i.bak 's|await import('\''\.\./\.\./\.\.'\'');|await import('\''/pkg/battleo.js'\'');|g' "$WORKER_FILE"
        
        # Fix the worker creation path
        echo "🔨 Fixing worker creation path..."
        sed -i.bak2 's|new Worker(new URL('\''\./workerHelpers\.js'\'', import\.meta\.url)|new Worker('\''/pkg/snippets/wasm-bindgen-rayon-38edf6e439f6d70d/src/workerHelpers.js'\'')|g' "$WORKER_FILE"
        
        # Fix syntax error (remove extra comma)
        echo "🔨 Fixing syntax error..."
        sed -i.bak3 's|new Worker('\''/pkg/snippets/wasm-bindgen-rayon-[^/]*/src/workerHelpers\.js'\''), {|new Worker('\''/pkg/snippets/wasm-bindgen-rayon-38edf6e439f6d70d/src/workerHelpers.js'\'', {|g' "$WORKER_FILE"
        
        # Clean up backup files
        rm -f "${WORKER_FILE}.bak" "${WORKER_FILE}.bak2" "${WORKER_FILE}.bak3"
        
        echo "✅ Rayon worker paths fixed successfully!"
    fi
    
    echo ""
    echo "📁 Generated files in pkg/:"
    ls -la pkg/
    echo ""
    echo "🎯 To test Rayon WASM functionality:"
    echo "   1. Start a local server: python3 -m http.server 8000"
    echo "   2. Open http://localhost:8000/index.html"
    echo "   3. Check browser console for Rayon initialization messages"
    echo ""
    echo "🔧 Build options used:"
    echo "   - Target: web"
    echo "   - Features: wasm-bindgen-rayon"
    echo "   - Optimization: disabled (for debugging)"
    echo "   - Worker import paths: automatically fixed"
else
    echo "❌ Build failed!"
    exit 1
fi 