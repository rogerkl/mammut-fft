import { BaseOperation, OperationConfig } from './BaseOperation';

export class SpectrumShiftOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Spectrum Shift',
      description: 'Shifts the entire frequency spectrum up or down by the specified amount in Hz.',
      controls: [
        {
          type: 'slider',
          id: 'shiftAmount',
          label: 'Shift Amount',
          min: -5000,
          max: 5000,
          step: 1,
          defaultValue: 0,
          unit: 'Hz',
          help: 'Positive values shift up, negative values shift down'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applySpectrumShift(values.shiftAmount);
  }
}

customElements.define('spectrum-shift-operation', SpectrumShiftOperation);