import { BaseOperation, OperationConfig } from './BaseOperation';

export class ThresholdOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Threshold',
      description: 'Apply a threshold filter to remove frequency components based on their amplitude.',
      controls: [
        {
          type: 'slider',
          id: 'level',
          label: 'Threshold Level',
          min: 0.01,
          max: 5.0,
          step: 0.01,
          defaultValue: 1.0,
        },
        {
          type: 'checkbox',
          id: 'removeAbove',
          label: 'Remove components above threshold (instead of below)',
          defaultValue: false,
          help: 'Check to remove frequencies above the threshold (peak limiter), uncheck to remove below (noise gate)'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyThreshold(values.level, values.removeAbove);
  }
}

customElements.define('threshold-operation', ThresholdOperation);