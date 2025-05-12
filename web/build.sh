#!/bin/bash
# Build script for the WebAssembly module

echo "Building mammut-fft FFT Audio Processor WebAssembly module..."

# Navigate to the web directory (if not already there)
cd "$(dirname "$0")"

# Build the library with wasm features
echo "Building library with WASM features..."
cargo build --release -p mammut_fft_lib --features wasm

# Build the WebAssembly module
echo "Building WASM module..."
wasm-pack build --release --target web

# Copy the pkg directory to www
echo "Copying WASM module to www directory..."
rm -rf www/pkg
cp -r pkg www/

echo "WASM build completed successfully"
echo "To run the web server: cd www && python3 -m http.server 8000"