import { BaseOperation, OperationConfig } from './BaseOperation';

export class ChannelSwapOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Channel Swap',
      description: 'Randomly swaps frequency bins between different channels. Only works with multi-channel audio (stereo or more).',
      controls: [
        {
          type: 'slider',
          id: 'repeat',
          label: 'Number of Swaps',
          min: 0.001,
          max: 10,
          step: 0.001,
          defaultValue: 0.1,
          unit: '%',
          help: 'Percentage of bins to swap between channels'
        }
      ]
    };
  }

  protected onApply(values: Record<string, any>): void {
    const info = this.audioService!.getInfo();
    if (info.channels < 2) {
      throw new Error('Channel swapping requires at least 2 channels (stereo audio)');
    }
    
    this.audioService!.swapChannels(values.repeat);
  }
}

customElements.define('channel-swap-operation', ChannelSwapOperation);