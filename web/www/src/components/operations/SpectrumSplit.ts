import { BaseOperation, OperationConfig } from './BaseOperation';

export class SpectrumSplitOperation extends BaseOperation {
  protected getConfig(): OperationConfig {
    return {
      name: 'Spectrum Split',
      description: 'Split the audio into multiple files, each containing only specific frequency bins. This allows separation of frequency components across multiple files.',
      controls: [
        {
          type: 'text',
          id: 'baseName',
          label: 'Base Filename',
          defaultValue: 'split',
          help: 'Base name for output files (e.g., "split" produces "split_0.wav", "split_1.wav", etc.)'
        },
        {
          type: 'number',
          id: 'numParts',
          label: 'Number of Parts',
          min: 2,
          max: 16,
          step: 1,
          defaultValue: 2,
          help: 'How many files to split the spectrum into'
        },
        {
          type: 'number',
          id: 'groupSize',
          label: 'Group Size',
          min: 1,
          max: 32,
          step: 1,
          defaultValue: 1,
          help: 'Number of consecutive frequency bins to group together'
        }
      ]
    };
  }

  protected async onApply(values: Record<string, any>): Promise<void> {
    const { baseName, numParts, groupSize } = values;
    
    // Show progress
    this.setProcessing(true, `Splitting into ${numParts} parts...`);
    
    try {
      // Process each part
      for (let i = 0; i < numParts; i++) {
        this.updateProgress(`Processing part ${i + 1}/${numParts}...`);
        
        // Prepare this part
        this.audioService!.prepareSplitPart(i, numParts, groupSize);
        
        // Process and download
        await this.audioService!.processAudio();
        
        const filename = `${baseName}_${i}.wav`;
        this.audioService!.downloadProcessedAudio(filename);
        
        // Reset for next part
        this.audioService!.resetSplit();
        
        // Small delay to prevent overwhelming the browser
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      
      this.updateProgress(`Split completed! Downloaded ${numParts} files.`);
    } finally {
      this.setProcessing(false);
    }
  }

  private setProcessing(processing: boolean, message?: string) {
    const button = this.root.querySelector('.apply-button') as HTMLButtonElement;
    if (button) {
      button.disabled = processing;
      if (message) {
        button.textContent = message;
      } else {
        button.textContent = `Apply ${this.getConfig().name}`;
      }
    }
  }

  private updateProgress(message: string) {
    // You could add a progress element to the base class
    const button = this.root.querySelector('.apply-button') as HTMLButtonElement;
    if (button) {
      button.textContent = message;
    }
  }
}

customElements.define('spectrum-split-operation', SpectrumSplitOperation);