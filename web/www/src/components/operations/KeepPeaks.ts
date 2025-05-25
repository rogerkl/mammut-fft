import { BaseOperation, OperationConfig } from './BaseOperation';

export class KeepPeaksOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Keep Peaks',
      description: 'Keeps only the local maxima in the frequency spectrum, zeroing out all other bins. Useful for isolating prominent frequency components.',
      controls: [] // No controls needed for this operation
    };
  }

  protected onApply(values: Record<string, any>): void {
    this.audioService!.keepPeaks();
  }
}

customElements.define('keep-peaks-operation', KeepPeaksOperation);