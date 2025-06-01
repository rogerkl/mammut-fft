import { BaseOperation, OperationConfig } from './BaseOperation';

export class PhaseMultiplyOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Phase Multiply',
      description: 'Multiply all phases with the value you specify. A value of -1 will reverse the sound.',
      controls: [
        {
          type: 'slider',
          id: 'factor',
          label: 'Phase Factor',
          min: -3,
          max: 3,
          step: 0.01,
          defaultValue: 1,
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.applyPhaseMultiply(values.factor);
  }
}

customElements.define('phase-multiply-operation', PhaseMultiplyOperation);