import { BaseOperation, OperationConfig } from './BaseOperation';

export class WobbleOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Wobble',
      description: 'Creates a sinusoidal modulation of frequency bins, resulting in a wobbling effect in the frequency domain.',
      controls: [
        {
          type: 'slider',
          id: 'frequency',
          label: 'Wobble Frequency',
          min: 1,
          max: 50,
          step: 0.5,
          defaultValue: 10,
          help: 'Controls the number of wobble cycles'
        },
        {
          type: 'slider',
          id: 'amplitude',
          label: 'Wobble Amplitude',
          min: 0.001,
          max: 0.1,
          step: 0.001,
          defaultValue: 0.01,
          help: 'Controls the displacement amount'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyWobble(values.frequency, values.amplitude);
  }
}

customElements.define('wobble-operation', WobbleOperation);