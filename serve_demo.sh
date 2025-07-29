#!/bin/bash

# HTTP server for Rayon WASM Demo with SharedArrayBuffer support
# This script serves the demo with proper CORS headers for SharedArrayBuffer

echo "🚀 Starting HTTP server for Rayon WASM Demo..."
echo ""
echo "📋 Important: This server includes CORS headers for SharedArrayBuffer support"
echo "🌐 Open your browser and navigate to: http://localhost:8000/rayon_demo.html"
echo ""
echo "⚠️  Note: SharedArrayBuffer requires cross-origin isolation policies"
echo "   The server is configured with proper headers for SharedArrayBuffer support"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Check if Node.js is available (preferred for CORS headers)
if command -v node &> /dev/null; then
    echo "🟢 Using Node.js HTTP server with CORS headers..."
    
    # Create a simple Node.js server with proper headers
    cat > server.js << 'EOF'
const http = require('http');
const fs = require('fs');
const path = require('path');

const mimeTypes = {
    '.html': 'text/html',
    '.js': 'application/javascript',
    '.wasm': 'application/wasm',
    '.json': 'application/json',
    '.css': 'text/css',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon'
};

const server = http.createServer((req, res) => {
    // Set CORS headers for SharedArrayBuffer support
    res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
    res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
    res.setHeader('Cross-Origin-Resource-Policy', 'same-origin');
    
    // Additional CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    let filePath = '.' + req.url;
    if (filePath === './') {
        filePath = './rayon_demo.html';
    }
    
    const extname = path.extname(filePath);
    const contentType = mimeTypes[extname] || 'application/octet-stream';
    
    fs.readFile(filePath, (error, content) => {
        if (error) {
            if (error.code === 'ENOENT') {
                res.writeHead(404);
                res.end('File not found');
            } else {
                res.writeHead(500);
                res.end('Server error: ' + error.code);
            }
        } else {
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(content, 'utf-8');
        }
    });
});

const PORT = 8000;
server.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}/`);
    console.log('SharedArrayBuffer headers configured for Rayon WASM support');
});
EOF

    node server.js

elif command -v python3 &> /dev/null; then
    echo "🐍 Using Python 3 HTTP server..."
    echo "⚠️  Warning: Python server may not support SharedArrayBuffer properly"
    echo "   Consider installing Node.js for better SharedArrayBuffer support"
    echo ""
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    echo "🐍 Using Python HTTP server..."
    echo "⚠️  Warning: Python server may not support SharedArrayBuffer properly"
    echo "   Consider installing Node.js for better SharedArrayBuffer support"
    echo ""
    python -m http.server 8000
else
    echo "❌ Neither Node.js nor Python found."
    echo ""
    echo "For proper SharedArrayBuffer support, please install Node.js:"
    echo "  - macOS: brew install node"
    echo "  - Ubuntu: sudo apt install nodejs npm"
    echo "  - Windows: Download from https://nodejs.org/"
    echo ""
    echo "Alternative options (may not support SharedArrayBuffer):"
    echo "  - PHP: php -S localhost:8000"
    echo "  - Any web server with CORS headers"
    exit 1
fi 