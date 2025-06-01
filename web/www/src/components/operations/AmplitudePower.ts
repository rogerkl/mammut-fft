import { BaseOperation, OperationConfig } from './BaseOperation';

export class AmplitudePowerOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Amplitude Power',
      description: 'Raises the amplitude of each frequency bin to the specified power.',
      controls: [
        {
          type: 'slider',
          id: 'power',
          label: 'Power Exponent',
          min: 0.01,
          max: 5,
          step: 0.01,
          defaultValue: 1,
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyPower(values.power);
  }
}

customElements.define('amplitude-power-operation', AmplitudePowerOperation);