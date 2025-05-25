console.log('Script loaded');

// Test basic DOM manipulation
document.addEventListener('DOMContentLoaded', () => {
  console.log('DOM loaded');
  
  const app = document.getElementById('app');
  if (app) {
    app.innerHTML = '<h1>Test - App is loading...</h1>';
    console.log('App element found and updated');
  } else {
    console.error('App element not found!');
  }
});

// Test WASM loading separately
async function testWasm() {
  try {
    console.log('Attempting to load WASM...');
    const wasmModule = await import('../pkg/mammut_fft_web.js');
    console.log('WASM module loaded:', wasmModule);
    
    const { default: init } = wasmModule;
    await init();
    console.log('WASM initialized successfully');
    
    document.getElementById('app')!.innerHTML = '<h1>WASM Loaded Successfully!</h1>';
  } catch (error) {
    console.error('WASM loading error:', error);
    document.getElementById('app')!.innerHTML = `<h1>Error: ${error}</h1>`;
  }
}

// Add a button to manually trigger WASM load
window.addEventListener('load', () => {
  const app = document.getElementById('app');
  if (app) {
    app.innerHTML = `
      <h1>Mammut FFT Test Page</h1>
      <button id="loadWasm">Load WASM Module</button>
      <div id="status"></div>
    `;
    
    document.getElementById('loadWasm')?.addEventListener('click', testWasm);
  }
});