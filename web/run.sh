#!/bin/bash
# Run script for the WebAssembly web server

echo "Starting web server for FFT Audio Processor..."

# Navigate to the web directory (if not already there)
cd "$(dirname "$0")"

# Check if the www/pkg directory exists
if [ ! -d "www/pkg" ]; then
  echo "WASM module not found in www/pkg. Building first..."
  ./build.sh
fi

# Navigate to www directory and start a web server
cd www
python3 -m http.server 8000