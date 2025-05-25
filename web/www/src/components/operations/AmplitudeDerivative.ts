import { BaseOperation, OperationConfig } from './BaseOperation';

export class AmplitudeDerivativeOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Amplitude Derivative',
      description: 'Replaces each frequency bin\'s amplitude with the difference between it and the previous bin, creating spectral edge detection effects.',
      controls: [
        {
          type: 'slider',
          id: 'multiplier',
          label: 'Derivative Multiplier',
          min: 0.1,
          max: 10.0,
          step: 0.1,
          defaultValue: 1.0,
          help: 'Scaling factor for the derivative values'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyAmplitudeDerivative(values.multiplier);
  }
}

customElements.define('amplitude-derivative-operation', AmplitudeDerivativeOperation);