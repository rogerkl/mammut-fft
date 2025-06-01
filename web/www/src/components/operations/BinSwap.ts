import { BaseOperation, OperationConfig } from './BaseOperation';

export class BinSwapOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Bin Swap',
      description: 'Randomly swaps frequency bins.',
      controls: [
        {
          type: 'slider',
          id: 'blockSize',
          label: 'Block Size',
          min: 0.01,
          max: 100,
          step: 0.01,
          defaultValue: 1,
          unit: '%',
          help: 'Maximum distance between swapped bins (% of full spectrum)'
        },
        {
          type: 'slider',
          id: 'repeat',
          label: 'Number of Swaps',
          min: 0.001,
          max: 10,
          step: 0.001,
          defaultValue: 0.1,
          unit: '%',
          help: 'Number of swaps to perform (% of number of bins)'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.swapBins(values.blockSize, values.repeat);
  }
}

customElements.define('bin-swap-operation', BinSwapOperation);