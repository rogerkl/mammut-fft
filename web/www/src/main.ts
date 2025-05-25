import './styles/main.css';
import { WasmService } from './services/WasmService';

// Import all components
import './components/AudioProcessorApp';
import './components/FileUploader';
import './components/AudioPlayer';
import './components/SpectrumVisualizer';
import './components/OperationTabs';
import './components/StatusBar';

import './components/operations/AmplitudePower';
import './components/operations/FrequencyFilters';
import './components/operations/PhaseOperations';
import './components/operations/BinSwap';
import './components/operations/ChannelSwap';
import './components/operations/SpectrumShift';
import './components/operations/FrequencyStretch';
import './components/operations/Wobble';
import './components/operations/Threshold';
import './components/operations/AmplitudeDerivative';
import './components/operations/KeepPeaks';
import './components/operations/SpectrumSplit';
import './components/operations/ChordFilter';
import './components/operations/Convolution';

// Initialize the application
async function init() {
  const appElement = document.getElementById('app');
  
  if (!appElement) {
    console.error('App element not found!');
    return;
  }

  try {
    // Show loading message
    appElement.innerHTML = '<div class="loading">Loading WASM module...</div>';
    
    // Wait for WASM to load
    console.log('Initializing WASM...');
    await WasmService.initialize();
    console.log('WASM initialized successfully');
    
    // Clear loading message
    appElement.innerHTML = '';
    
    // Create and mount the main app
    const app = document.createElement('audio-processor-app');
    appElement.appendChild(app);
    
    console.log('App component mounted');
  } catch (error) {
    console.error('Failed to initialize application:', error);
    appElement.innerHTML = `
      <div class="error">
        <h1>Failed to load application</h1>
        <p>Error: ${error}</p>
        <p>Check the console for more details.</p>
      </div>
    `;
  }
}

// Start the app when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}