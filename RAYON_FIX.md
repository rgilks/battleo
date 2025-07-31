# Rayon Worker Import Fix

## Problem

When using `wasm-bindgen-rayon` with WebAssembly, the generated worker files use relative import paths that fail when served from a web server. The error typically looks like:

```
workerHelpers.js:1 Uncaught (in promise) TypeError: Failed to fetch dynamically imported module: http://localhost:8000/pkg/
```

This happens because the worker files are generated in a subdirectory structure like:

```
pkg/
├── battleo.js
├── battleo_bg.wasm
└── snippets/
    └── wasm-bindgen-rayon-[hash]/
        └── src/
            └── workerHelpers.js
```

The `workerHelpers.js` file tries to import the main module using a relative path `../../..`, but this doesn't work correctly when served from a web server.

## Solution

This fix provides a robust solution by automatically fixing the worker import paths during the build process:

1. **Integrated Build Scripts**: Both `build.sh` and `build_with_rayon.sh` now automatically detect and fix Rayon worker import issues
2. **Direct Path Fixes**: Modifies the generated worker files to use absolute paths instead of relative paths
3. **Automatic Detection**: Detects when Rayon workers are present and applies fixes automatically

## Usage

### Automatic Fix (Recommended)

Simply use the build scripts as normal - the fix is applied automatically:

```bash
# For Rayon builds
./build_with_rayon.sh

# For regular builds (also fixes Rayon if detected)
./build.sh
```

The build scripts will:

1. Build the WebAssembly module
2. Automatically detect if Rayon workers were generated
3. Fix the import paths in the worker files
4. Provide feedback on what was fixed

### What Gets Fixed

The build scripts automatically fix:

1. **Import Path**: Changes `await import('../../..')` to `await import('/pkg/battleo.js')`
2. **Worker Creation Path**: Updates worker creation to use absolute paths
3. **Syntax Errors**: Fixes any syntax issues in the generated worker files

### Testing the Fix

1. Start a local server:

   ```bash
   python3 -m http.server 8000
   ```

2. Open the application:

   ```
   http://localhost:8000/index.html
   ```

3. Check the browser console for Rayon initialization messages

## How It Works

### Original Problem

The generated `workerHelpers.js` file contains:

```javascript
const pkg = await import("../../..");
```

This relative path fails when served from a web server.

### Fixed Version

The build script automatically changes it to:

```javascript
const pkg = await import("/pkg/battleo.js");
```

This ensures the worker can always find the main WASM module regardless of the URL structure.

### Build Script Integration

The fix is now integrated directly into the build process:

```bash
# During build, the script automatically:
# 1. Detects Rayon worker files
# 2. Creates backups of original files
# 3. Applies path fixes using sed
# 4. Cleans up temporary files
# 5. Reports success/failure
```

## Browser Compatibility

This fix works with all modern browsers that support:

- ES6 modules
- Web Workers
- Dynamic imports
- URL API

## Troubleshooting

### Still Getting Import Errors

1. Make sure you're serving the files from a web server (not opening the HTML file directly)
2. Check that the `pkg/` directory is accessible from the web server
3. Verify that the build script ran successfully and reported "Rayon worker paths fixed!"

### Worker Count Issues

If you're getting worker count errors, make sure:

1. The browser supports Web Workers
2. You're not running in a restricted environment (like some corporate networks)
3. The browser allows multiple workers

### Performance Issues

If you experience performance issues:

1. Reduce the number of workers (default is 4)
2. Check browser console for any errors
3. Consider using the fallback mode for debugging

## Fallback Mode

The original code includes a fallback mode that doesn't use parallel processing. This is automatically used if Rayon initialization fails:

```javascript
try {
  await parallelProcessor.initialize();
  // Rayon mode
} catch (error) {
  await parallelProcessor.initialize_fallback();
  // Fallback mode (no parallel processing)
}
```

## Build Scripts

### build_with_rayon.sh

- Builds with `wasm-bindgen-rayon` feature enabled
- Uses nightly Rust toolchain
- Automatically fixes worker import paths
- Provides detailed feedback

### build.sh

- Standard build process
- Automatically detects and fixes Rayon workers if present
- Works with or without Rayon features

## Contributing

If you encounter issues with this fix:

1. Check the browser console for specific error messages
2. Verify that the build script reported successful fixes
3. Check that all files are being served correctly
4. Ensure you're using a web server (not file:// URLs)

## License

This fix is provided as-is to solve the specific issue with `wasm-bindgen-rayon` worker imports. The original `wasm-bindgen-rayon` code is licensed under the Apache License 2.0.
