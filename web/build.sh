#!/bin/bash
# Build script for the WebAssembly module and TypeScript app

echo "Building mammut-fft FFT Audio Processor..."

# Navigate to the web directory
cd "$(dirname "$0")"

# Build the library with wasm features
echo "Building library with WASM features..."
cargo build --release -p mammut_fft_lib --features wasm

# Build the WebAssembly module
echo "Building WASM module..."
wasm-pack build --release --target web --out-dir www/pkg

# Navigate to www directory
cd www

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
  echo "Installing dependencies..."
  npm install
fi

# Build the TypeScript application
echo "Building TypeScript application..."
npm run build

echo "Build completed successfully!"
echo "To run the development server: cd www && npm run dev"
echo "Production files are in: www/dist/"