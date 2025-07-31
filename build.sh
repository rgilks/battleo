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
    
    # Check if Rayon workers were generated and fix them if needed
    WORKER_FILE=$(find pkg/snippets -name "workerHelpers.js" -type f 2>/dev/null | head -n 1)
    
    if [ -n "$WORKER_FILE" ]; then
        echo "🔧 Detected Rayon workers - fixing import paths..."
        
        # Create backup
        cp "$WORKER_FILE" "${WORKER_FILE}.backup"
        
        # Fix the import path to use absolute path
        sed -i.bak 's|await import('\''\.\./\.\./\.\.'\'');|await import('\''/pkg/battleo.js'\'');|g' "$WORKER_FILE"
        
        # Fix the worker creation path
        sed -i.bak2 's|new Worker(new URL('\''\./workerHelpers\.js'\'', import\.meta\.url)|new Worker('\''/pkg/snippets/wasm-bindgen-rayon-38edf6e439f6d70d/src/workerHelpers.js'\'')|g' "$WORKER_FILE"
        
        # Fix syntax error (remove extra comma)
        sed -i.bak3 's|new Worker('\''/pkg/snippets/wasm-bindgen-rayon-[^/]*/src/workerHelpers\.js'\''), {|new Worker('\''/pkg/snippets/wasm-bindgen-rayon-38edf6e439f6d70d/src/workerHelpers.js'\'', {|g' "$WORKER_FILE"
        
        # Clean up backup files
        rm -f "${WORKER_FILE}.bak" "${WORKER_FILE}.bak2" "${WORKER_FILE}.bak3"
        
        echo "✅ Rayon worker paths fixed!"
    fi
    
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