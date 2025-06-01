import { BaseOperation, OperationConfig } from './BaseOperation';

export class FrequencyStretchOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Stretch',
      description: 'Applies non-linear stretching to the frequency spectrum.',
      controls: [
        {
          type: 'slider',
          id: 'exponent',
          label: 'Stretch Exponent',
          min: 0.2,
          max: 5,
          step: 0.1,
          defaultValue: 1.3,
          help: 'Higher values compress high frequencies more'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyStretch(values.exponent);
  }
}

customElements.define('frequency-stretch-operation', FrequencyStretchOperation);