import { BaseOperation, OperationConfig } from './BaseOperation';

export class WobbleOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Wobble',
      description: 'This transform will alternately stretch and contract the frequency axis using a sinusoidal transfer function for the frequencies._The Frequency parameter controls the number of periods of the transfer function from 0 Hz to the Nyquist frequency, while Amplitude controls its amplitude (1 is the entire frequency axis).',
      controls: [
        {
          type: 'slider',
          id: 'frequency',
          label: 'Frequency',
          min: 1,
          max: 5000,
          step: 1.,
          defaultValue: 10,
          help: 'Number of periods of the transfer function'
        },
        {
          type: 'slider',
          id: 'amplitude',
          label: 'Amplitude',
          min: 0.001,
          max: 0.1,
          step: 0.001,
          defaultValue: 0.01,
          help: 'Amount'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyWobble(values.frequency, values.amplitude);
  }
}

customElements.define('wobble-operation', WobbleOperation);