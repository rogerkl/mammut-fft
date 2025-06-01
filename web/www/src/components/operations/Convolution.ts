import { BaseOperation, OperationConfig } from './BaseOperation';

export class ConvolutionOperation extends BaseOperation {
  private convolveFile: File | null = null;

  protected getConfig(): OperationConfig {
    return {
      name: 'Convolution',
      description: 'Convolve or correlate your audio with another audio file.',
      controls: [
        {
          type: 'select',
          id: 'mode',
          label: 'Operation Mode',
          defaultValue: 'convolve',
          options: [
            { value: 'convolve', label: 'Convolution' },
            { value: 'correlate', label: 'Correlation' }
          ],
          help: 'Convolution time-reverses the IR before multiplying spectra'
        },
        {
          type: 'slider',
          id: 'wetMix',
          label: 'Wet/Dry Mix',
          min: 0,
          max: 1,
          step: 0.01,
          defaultValue: 1,
          help: '0 = 100% dry (original), 1 = 100% wet (processed)'
        }
      ]
    };
  }

  connectedCallback() {
    super.connectedCallback();
    this.addFileInput();
  }

  private addFileInput() {
    const controlsContainer = this.root.querySelector('.controls-container');
    if (controlsContainer) {
      const fileInputHTML = `
        <div class="form-group file-input-group">
          <label for="convolveFile">Select IR/Second File:</label>
          <div class="file-input-wrapper">
            <input type="file" id="convolveFile" accept="audio/*" style="display: none;">
            <button class="file-select-button" id="selectFileButton">Choose File</button>
            <span id="fileInfo" class="file-info">No file selected</span>
          </div>
        </div>
      `;
      
      // Insert before the first control
      controlsContainer.insertAdjacentHTML('afterbegin', fileInputHTML);
      
      // Setup file input events
      const fileInput = this.root.getElementById('convolveFile') as HTMLInputElement;
      const selectButton = this.root.getElementById('selectFileButton');
      const fileInfo = this.root.getElementById('fileInfo');
      
      selectButton?.addEventListener('click', () => fileInput.click());
      
      fileInput.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
          this.convolveFile = target.files[0];
          const fileSize = (this.convolveFile.size / 1024).toFixed(1);
          fileInfo!.textContent = `${this.convolveFile.name} (${fileSize} KB)`;
        }
      });
    }
  }

  protected async onApply(values: Record<string, any>): Promise<void> {
    if (!this.convolveFile) {
      throw new Error('Please select a file for convolution');
    }

    const arrayBuffer = await this.convolveFile.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);
    
    const isCorrelate = values.mode === 'correlate';
    await this.audioService!.convolveWithFile(uint8Array, isCorrelate, values.wetMix);
  }

  protected getStyles(): string {
    return super.getStyles() + `
      .file-input-group {
        margin-bottom: 15px;
        padding-bottom: 15px;
        border-bottom: 1px solid var(--border-color, #333);
      }
      
      .file-input-wrapper {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-top: 8px;
      }
      
      .file-select-button {
        padding: 8px 16px;
        background-color: #444;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
        transition: background-color 0.2s;
      }
      
      .file-select-button:hover {
        background-color: #555;
      }
      
      .file-info {
        font-size: 13px;
        color: var(--text-secondary, #aaa);
      }
    `;
  }
}

customElements.define('convolution-operation', ConvolutionOperation);