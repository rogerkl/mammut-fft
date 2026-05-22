#!/bin/bash
# Build script for the WebAssembly module and TypeScript app
set -euo pipefail

echo "Building mammut-fft FFT Audio Processor..."

# Navigate to the web directory
cd "$(dirname "$0")"

# Required tools
missing=()
for tool in cargo wasm-pack npm; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        missing+=("$tool")
    fi
done
if [ ${#missing[@]} -gt 0 ]; then
    echo "Error: missing required tool(s): ${missing[*]}" >&2
    echo "" >&2
    for tool in "${missing[@]}"; do
        case "$tool" in
            cargo)     echo "  cargo:     install via https://rustup.rs" >&2 ;;
            wasm-pack) echo "  wasm-pack: cargo install wasm-pack   (or https://rustwasm.github.io/wasm-pack/installer/)" >&2 ;;
            npm)      echo "  npm:       install Node.js (https://nodejs.org)" >&2 ;;
        esac
    done
    exit 1
fi

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