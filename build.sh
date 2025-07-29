#!/bin/bash

echo "🚀 Building Battleo Simulation..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build the WebAssembly module
echo "📦 Building WebAssembly module..."
wasm-pack build --target web

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🌐 To run the simulation:"
    echo "   python3 -m http.server 8000"
    echo "   Then open http://localhost:8000 in your browser"
    echo ""
    echo "🔧 Development tips:"
    echo "   - Use browser dev tools to monitor performance"
    echo "   - Check console for any errors"
    echo "   - Agent count affects performance significantly"
else
    echo "❌ Build failed!"
    exit 1
fi 