import { AudioProcessorService } from '../services/AudioProcessor';

// Import all operation components
import './operations/AmplitudePower';
import './operations/FrequencyFilters';
import './operations/PhaseOperations';
import './operations/BinSwap';
import './operations/ChannelSwap';
import './operations/SpectrumShift';
import './operations/FrequencyStretch';
import './operations/Wobble';
import './operations/Threshold';
import './operations/AmplitudeDerivative';
import './operations/KeepPeaks';
import './operations/SpectrumSplit';
import './operations/ChordFilter';
import './operations/Convolution';

interface TabConfig {
  id: string;
  label: string;
  component: string;
}

export class OperationTabs extends HTMLElement {
  private shadowRoot: ShadowRoot;
  private audioService: AudioProcessorService | null = null;
  private activeTab: string = 'power';

  private tabs: TabConfig[] = [
    { id: 'power', label: 'Amplitude Power', component: 'amplitude-power-operation' },
    { id: 'phase-mult', label: 'Phase Multiplication', component: 'phase-multiply-operation' },
    { id: 'bin-swap', label: 'Frequency Bin Swap', component: 'bin-swap-operation' },
    { id: 'channel-swap', label: 'Channel Bin Swap', component: 'channel-swap-operation' },
    { id: 'shift', label: 'Spectrum Shift', component: 'spectrum-shift-operation' },
    { id: 'stretch', label: 'Stretch', component: 'frequency-stretch-operation' },
    { id: 'wobble', label: 'Wobble', component: 'wobble-operation' },
    { id: 'threshold', label: 'Threshold', component: 'threshold-operation' },
    { id: 'derivate', label: 'Derivative Amplitude', component: 'amplitude-derivative-operation' },
    { id: 'keep-peaks', label: 'Keep Peaks', component: 'keep-peaks-operation' },
    { id: 'split', label: 'Frequency Spectrum Split', component: 'spectrum-split-operation' },
    { id: 'filters', label: 'Filters', component: 'frequency-filters-operation' },
    { id: 'chord-filter', label: 'Chord Filter', component: 'chord-filter-operation' },
    { id: 'convolution', label: 'Convolution', component: 'convolution-operation' }  
  ];

  constructor() {
    super();
    this.shadowRoot = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    this.setupEventListeners();
  }

  setAudioService(service: AudioProcessorService) {
    this.audioService = service;
    
    // Update all operation components
    this.tabs.forEach(tab => {
      const component = this.shadowRoot.querySelector(tab.component) as any;
      component?.setAudioService(service);
    });
  }

  private render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          margin: 30px 0;
        }
        
        .tabs-container {
          background-color: var(--card-bg, #1e1e1e);
          border-radius: 8px;
          overflow: hidden;
          box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }
        
        .tabs-header {
          display: flex;
          flex-wrap: wrap;
          background-color: rgba(0, 0, 0, 0.2);
          border-bottom: 1px solid var(--border-color, #333);
        }
        
        .tab-button {
          flex: 1 0 auto;
          min-width: 120px;
          padding: 12px 20px;
          background: none;
          border: none;
          color: var(--text-secondary, #aaa);
          font-size: 15px;
          cursor: pointer;
          transition: all 0.3s ease;
          position: relative;
        }
        
        .tab-button:hover {
          background-color: rgba(76, 175, 80, 0.1);
          color: var(--text-primary, #e0e0e0);
        }
        
        .tab-button.active {
          color: white;
          font-weight: 500;
          background-color: var(--success-color, #4CAF50);
        }
        
        .tab-button.active::after {
          content: '';
          position: absolute;
          bottom: 0;
          left: 0;
          width: 100%;
          height: 3px;
          background-color: var(--success-color, #4CAF50);
        }
        
        .tabs-content {
          padding: 20px;
        }
        
        .tab-content {
          display: none;
        }
        
        .tab-content.active {
          display: block;
        }
      </style>
      
      <div class="tabs-container">
        <div class="tabs-header">
          ${this.tabs.map(tab => `
            <button class="tab-button ${tab.id === this.activeTab ? 'active' : ''}" 
                    data-tab="${tab.id}">
              ${tab.label}
            </button>
          `).join('')}
        </div>
        
        <div class="tabs-content">
          ${this.tabs.map(tab => `
            <div class="tab-content ${tab.id === this.activeTab ? 'active' : ''}" 
                 id="${tab.id}-tab">
              <${tab.component}></${tab.component}>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  private setupEventListeners() {
    const tabButtons = this.shadowRoot.querySelectorAll('.tab-button');
    
    tabButtons.forEach(button => {
      button.addEventListener('click', (e) => {
        const target = e.target as HTMLButtonElement;
        const tabId = target.dataset.tab!;
        this.switchTab(tabId);
      });
    });
  }

  private switchTab(tabId: string) {
    this.activeTab = tabId;
    
    // Update button states
    this.shadowRoot.querySelectorAll('.tab-button').forEach(button => {
      button.classList.toggle('active', button.getAttribute('data-tab') === tabId);
    });
    
    // Update content visibility
    this.shadowRoot.querySelectorAll('.tab-content').forEach(content => {
      content.classList.toggle('active', content.id === `${tabId}-tab`);
    });
  }
}

customElements.define('operation-tabs', OperationTabs);