import { AudioProcessorService } from '../../services/AudioProcessor';

export interface OperationConfig {
  name: string;
  description: string;
  controls: ControlConfig[];
}

export interface ControlConfig {
  type: 'slider' | 'select' | 'checkbox' | 'number' | 'text';
  id: string;
  label: string;
  min?: number;
  max?: number;
  step?: number;
  defaultValue?: any;
  options?: { value: string; label: string }[];
  unit?: string;
  help?: string;
}

export abstract class BaseOperation extends HTMLElement {
  protected root: ShadowRoot;
  protected audioService: AudioProcessorService | null = null;
  protected controls: Map<string, HTMLElement> = new Map();

  constructor() {
    super();
    this.root = this.attachShadow({ mode: 'open' });
  }

  // Abstract method that child classes must implement
  protected abstract getConfig(): OperationConfig;
  protected abstract onApply(values: Record<string, any>): void;

  connectedCallback() {
    this.render();
    this.setupEventListeners();
  }

  setAudioService(service: AudioProcessorService) {
    this.audioService = service;
  }

  private render() {
    const config = this.getConfig();
    
    this.root.innerHTML = `
      <style>
        ${this.getStyles()}
      </style>
      
      <div class="operation-content">
        <p class="description">${config.description}</p>
        
        <div class="controls-container">
          ${config.controls.map(control => this.renderControl(control)).join('')}
        </div>
        
        <button class="apply-button" id="applyButton">Apply ${config.name}</button>
      </div>
    `;

    // Store references to controls
    config.controls.forEach(control => {
      const element = this.root.getElementById(control.id);
      if (element) {
        this.controls.set(control.id, element);
      }
    });
  }

  private renderControl(control: ControlConfig): string {
    switch (control.type) {
      case 'slider':
        return this.renderSlider(control);
      case 'select':
        return this.renderSelect(control);
      case 'checkbox':
        return this.renderCheckbox(control);
      case 'number':
        return this.renderNumberInput(control);
      case 'text':
        return this.renderTextInput(control);
      default:
        return '';
    }
  }

  private renderSlider(control: ControlConfig): string {
    const value = control.defaultValue ?? control.min ?? 0;
    const displayValue = this.formatValue(value, control.unit);
    
    return `
      <div class="slider-container">
        <div class="slider-label">
          <span>${control.label}:</span>
          <span class="value-display" id="${control.id}Value">${displayValue}</span>
        </div>
        ${control.help ? `<div class="help-text">${control.help}</div>` : ''}
        <input 
          type="range" 
          id="${control.id}" 
          min="${control.min ?? 0}" 
          max="${control.max ?? 100}" 
          step="${control.step ?? 1}" 
          value="${value}"
          data-unit="${control.unit || ''}"
        >
      </div>
    `;
  }

  private renderSelect(control: ControlConfig): string {
    return `
      <div class="form-group">
        <label for="${control.id}">${control.label}:</label>
        ${control.help ? `<div class="help-text">${control.help}</div>` : ''}
        <select id="${control.id}">
          ${control.options?.map(option => `
            <option value="${option.value}" ${option.value === control.defaultValue ? 'selected' : ''}>
              ${option.label}
            </option>
          `).join('')}
        </select>
      </div>
    `;
  }

  private renderCheckbox(control: ControlConfig): string {
    return `
      <div class="form-group checkbox-group">
        <label>
          <input 
            type="checkbox" 
            id="${control.id}" 
            ${control.defaultValue ? 'checked' : ''}
          >
          ${control.label}
        </label>
        ${control.help ? `<div class="help-text">${control.help}</div>` : ''}
      </div>
    `;
  }

  private renderNumberInput(control: ControlConfig): string {
    return `
      <div class="form-group">
        <label for="${control.id}">${control.label}:</label>
        ${control.help ? `<div class="help-text">${control.help}</div>` : ''}
        <input 
          type="number" 
          id="${control.id}" 
          min="${control.min ?? ''}" 
          max="${control.max ?? ''}" 
          step="${control.step ?? 1}" 
          value="${control.defaultValue ?? ''}"
          class="number-input"
        >
      </div>
    `;
  }

  private renderTextInput(control: ControlConfig): string {
    return `
      <div class="form-group">
        <label for="${control.id}">${control.label}:</label>
        ${control.help ? `<div class="help-text">${control.help}</div>` : ''}
        <input 
          type="text" 
          id="${control.id}" 
          value="${control.defaultValue ?? ''}"
          class="text-input"
        >
      </div>
    `;
  }

  private setupEventListeners() {
    const config = this.getConfig();

    // Setup value display updates for sliders
    config.controls.forEach(control => {
      if (control.type === 'slider') {
        const slider = this.controls.get(control.id) as HTMLInputElement;
        const valueDisplay = this.root.getElementById(`${control.id}Value`);
        
        if (slider && valueDisplay) {
          slider.addEventListener('input', () => {
            const value = parseFloat(slider.value);
            valueDisplay.textContent = this.formatValue(value, control.unit);
          });
        }
      }
    });

    // Setup apply button
    const applyButton = this.root.getElementById('applyButton');
    applyButton?.addEventListener('click', () => {
      if (!this.audioService) {
        console.error('Audio service not set');
        return;
      }

      // Collect all control values
      const values: Record<string, any> = {};
      
      config.controls.forEach(control => {
        const element = this.controls.get(control.id);
        if (!element) return;

        switch (control.type) {
          case 'slider':
          case 'number':
            values[control.id] = parseFloat((element as HTMLInputElement).value);
            break;
          case 'checkbox':
            values[control.id] = (element as HTMLInputElement).checked;
            break;
          case 'select':
          case 'text':
            values[control.id] = (element as HTMLInputElement).value;
            break;
        }
      });

      try {
        this.onApply(values);
        this.dispatchEvent(new CustomEvent('operationApplied', {
          detail: { operation: config.name, values },
          bubbles: true,
          composed: true
        }));
      } catch (error) {
        console.error(`Error applying ${config.name}:`, error);
      }
    });
  }

  protected formatValue(value: number, unit?: string): string {
    if (unit === 'Hz' && value >= 1000) {
      return `${(value / 1000).toFixed(1)} kHz`;
    }
    if (unit === '%') {
      return `${value}%`;
    }
    if (unit) {
      return `${value} ${unit}`;
    }
    return value.toString();
  }

  private getStyles(): string {
    return `
      :host {
        display: block;
      }
      
      .operation-content {
        padding: 20px;
        background-color: rgba(255, 255, 255, 0.03);
        border-radius: 8px;
      }
      
      .description {
        font-size: 14px;
        color: var(--text-secondary, #aaa);
        margin-bottom: 20px;
        line-height: 1.4;
      }
      
      .controls-container {
        display: flex;
        flex-direction: column;
        gap: 15px;
      }
      
      /* Form groups */
      .form-group {
        display: flex;
        flex-direction: column;
        gap: 5px;
      }
      
      .checkbox-group {
        flex-direction: row;
        align-items: center;
      }
      
      .checkbox-group label {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
      }
      
      label {
        font-size: 14px;
        color: var(--text-secondary, #aaa);
      }
      
      .help-text {
        font-size: 12px;
        color: var(--text-secondary, #888);
        font-style: italic;
      }
      
      /* Slider styles */
      .slider-container {
        margin: 15px 0;
      }
      
      .slider-label {
        display: flex;
        justify-content: space-between;
        margin-bottom: 8px;
      }
      
      .value-display {
        color: var(--accent-color, #4f9eff);
        font-weight: 500;
      }
      
      input[type="range"] {
        width: 100%;
        height: 6px;
        -webkit-appearance: none;
        background-color: var(--slider-track, #444);
        border-radius: 3px;
        outline: none;
      }
      
      input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        width: 16px;
        height: 16px;
        background-color: var(--slider-thumb, #6fb5ff);
        border-radius: 50%;
        cursor: pointer;
        transition: background-color 0.2s;
      }
      
      input[type="range"]::-moz-range-thumb {
        width: 16px;
        height: 16px;
        background-color: var(--slider-thumb, #6fb5ff);
        border: none;
        border-radius: 50%;
        cursor: pointer;
      }
      
      /* Select styles */
      select {
        padding: 8px 12px;
        background-color: rgba(0, 0, 0, 0.2);
        color: var(--text-primary, #e0e0e0);
        border: 1px solid var(--border-color, #333);
        border-radius: 4px;
        font-size: 14px;
        cursor: pointer;
      }
      
      /* Input styles */
      input[type="text"],
      input[type="number"] {
        padding: 8px 12px;
        background-color: rgba(0, 0, 0, 0.2);
        color: var(--text-primary, #e0e0e0);
        border: 1px solid var(--border-color, #333);
        border-radius: 4px;
        font-size: 14px;
      }
      
      input[type="checkbox"] {
        width: 18px;
        height: 18px;
        cursor: pointer;
      }
      
      /* Button styles */
      .apply-button {
        margin-top: 20px;
        padding: 12px 24px;
        background-color: var(--accent-color, #4f9eff);
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
        font-weight: 500;
        transition: all 0.2s;
      }
      
      .apply-button:hover {
        background-color: var(--accent-hover, #3d7dcf);
        transform: translateY(-1px);
      }
      
      .apply-button:active {
        transform: translateY(0);
      }
      
      .apply-button:disabled {
        background-color: var(--border-color, #333);
        cursor: not-allowed;
        opacity: 0.7;
      }
    `;
  }
}