import { BaseOperation, OperationConfig } from './BaseOperation';

export class FrequencyFiltersOperation extends BaseOperation {
  private filterType: string = 'lowpass';

  protected getConfig(): OperationConfig {
    return {
      name: 'Frequency Filter',
      description: 'Apply various filters to remove or isolate specific frequency ranges in your audio.',
      controls: [
        {
          type: 'select',
          id: 'filterType',
          label: 'Filter Type',
          defaultValue: 'lowpass',
          options: [
            { value: 'lowpass', label: 'Lowpass' },
            { value: 'highpass', label: 'Highpass' },
            { value: 'bandpass', label: 'Bandpass' }
          ]
        },
        {
          type: 'slider',
          id: 'cutoffLow',
          label: 'Cutoff Frequency',
          min: 20,
          max: 20000,
          step: 1,
          defaultValue: 1000,
          unit: 'Hz'
        },
        {
          type: 'slider',
          id: 'cutoffHigh',
          label: 'High Cutoff',
          min: 20,
          max: 20000,
          step: 1,
          defaultValue: 5000,
          unit: 'Hz'
        }
      ]
    };
  }

  connectedCallback() {
    super.connectedCallback();
    this.setupFilterTypeHandling();
  }

  private setupFilterTypeHandling() {
    const filterSelect = this.controls.get('filterType') as HTMLSelectElement;
    const highCutoffContainer = this.root.querySelector('[id="cutoffHigh"]')?.closest('.slider-container');
    const lowCutoffLabel = this.root.querySelector('[for="cutoffLow"]');

    if (filterSelect && highCutoffContainer) {
      // Initially hide high cutoff
      (highCutoffContainer as HTMLElement).style.display = 'none';

      filterSelect.addEventListener('change', () => {
        this.filterType = filterSelect.value;
        
        if (this.filterType === 'bandpass') {
          (highCutoffContainer as HTMLElement).style.display = 'block';
          if (lowCutoffLabel) {
            lowCutoffLabel.textContent = 'Low Cutoff:';
          }
        } else {
          (highCutoffContainer as HTMLElement).style.display = 'none';
          if (lowCutoffLabel) {
            lowCutoffLabel.textContent = 'Cutoff Frequency:';
          }
        }
      });
    }
  }

  protected onApply(values: Record<string, any>): void {
    switch (values.filterType) {
      case 'lowpass':
        this.audioService!.applyLowpass(values.cutoffLow);
        break;
      case 'highpass':
        this.audioService!.applyHighpass(values.cutoffLow);
        break;
      case 'bandpass':
        this.audioService!.applyBandpass(values.cutoffLow, values.cutoffHigh);
        break;
    }
  }
}

customElements.define('frequency-filters-operation', FrequencyFiltersOperation);