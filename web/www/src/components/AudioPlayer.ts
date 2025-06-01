export class AudioPlayer extends HTMLElement {
  private root: ShadowRoot;
  private audio!: HTMLAudioElement;
  private resetButton!: HTMLButtonElement;
  private downloadButton!: HTMLButtonElement;
  private isProcessed: boolean = false;
  private isProcessing: boolean = false;

  constructor() {
    super();
    this.root = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    this.setupEventListeners();
  }

  private render() {
    this.root.innerHTML = `
      <style>
        :host {
          display: block;
          margin: 25px 0;
        }
        
        .audio-panel {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 20px;
          background-color: rgba(255, 255, 255, 0.05);
          padding: 15px;
          border-radius: 8px;
          flex-wrap: wrap;
        }
        
        audio {
          width: 60%;
          min-width: 300px;
          height: 40px;
          background-color: rgba(0, 0, 0, 0.2);
          border-radius: 4px;
          outline: none;
        }
        
        .button-group {
          display: flex;
          gap: 10px;
          flex-wrap: wrap;
        }
        
        button {
          padding: 10px 20px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
          color: white;
        }
        
        .reset-button {
          background-color: #666;
        }
        
        .reset-button:hover {
          background-color: #777;
        }
        
        .download-button {
          background-color: var(--accent-color, #4f9eff);
        }
        
        .download-button:hover {
          background-color: var(--accent-hover, #3d7dcf);
        }
        
        button:disabled {
          background-color: var(--border-color, #333);
          cursor: not-allowed;
          opacity: 0.7;
        }
        
        .processing {
          opacity: 0.7;
          pointer-events: none;
        }
        
        @media (max-width: 768px) {
          .audio-panel {
            flex-direction: column;
          }
          
          audio {
            width: 100%;
          }
          
          .button-group {
            width: 100%;
            justify-content: space-between;
          }
        }
      </style>
      
      <div class="audio-panel">
        <audio id="audioPlayer" controls></audio>
        
        <div class="button-group">
          <button class="reset-button" id="resetButton">Reset to Original</button>
          <button class="download-button" id="downloadButton">Download Processed</button>
        </div>
      </div>
    `;

    this.audio = this.root.getElementById('audioPlayer') as HTMLAudioElement;
    this.resetButton = this.root.getElementById('resetButton') as HTMLButtonElement;
    this.downloadButton = this.root.getElementById('downloadButton') as HTMLButtonElement;
  }

  private setupEventListeners() {
    // Auto-process when play button is clicked
    this.audio.addEventListener('play', async (event) => {
      if (!this.isProcessed && !this.isProcessing) {
        // Pause the audio immediately to prevent playing unprocessed audio
        this.audio.pause();

        // Process the audio first
        await this.processAudio();

        // Resume playing after processing is complete
        if (this.isProcessed) {
          this.audio.play();
        }
      }
    });

    this.resetButton.addEventListener('click', () => {
      this.resetAudio();
    });

    this.downloadButton.addEventListener('click', () => {
      this.dispatchEvent(new Event('downloadAudio', { bubbles: true, composed: true }));
    });
  }

  private async processAudio(): Promise<void> {
    if (this.isProcessing) return;

    this.isProcessing = true;
    this.audio.classList.add('processing');

    try {
      // Dispatch the processing event and wait for completion
      const processEvent = new CustomEvent('processAudio', {
        bubbles: true,
        composed: true,
        detail: { callback: this.onProcessingComplete.bind(this) }
      });
      this.dispatchEvent(processEvent);

    } catch (error) {
      console.error('Processing failed:', error);
      this.onProcessingComplete(false);
    }
  }

  private onProcessingComplete(success: boolean = true): void {
    this.isProcessing = false;
    this.isProcessed = success;
    this.audio.classList.remove('processing');
  }

  private resetAudio(): void {
    this.isProcessed = false;
    this.dispatchEvent(new Event('resetAudio', { bubbles: true, composed: true }));
  }

  updateAudio(audioData: ArrayBuffer) {
    const blob = new Blob([audioData], { type: 'audio/wav' });
    const url = URL.createObjectURL(blob);

    // Clean up old URL
    if (this.audio.src && this.audio.src.startsWith('blob:')) {
      URL.revokeObjectURL(this.audio.src);
    }

    this.audio.src = url;

    // Mark as processed if this is updating with processed audio
    this.onProcessingComplete(true);
  }

  setEnabled(enabled: boolean) {
    this.resetButton.disabled = !enabled;
    this.downloadButton.disabled = !enabled;
  }

  // Public method to set original/unprocessed audio
  resetProcessed() {
    this.isProcessed = false;
  }

  setOriginalAudio(audioData: ArrayBuffer) {
    this.isProcessed = false;
    this.updateAudio(audioData);
  }
}

customElements.define('audio-player', AudioPlayer);