export class AudioPlayer extends HTMLElement {
  private root: ShadowRoot;
  private audio!: HTMLAudioElement;
  private processButton!: HTMLButtonElement;
  private resetButton!: HTMLButtonElement;
  private downloadButton!: HTMLButtonElement;

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
        
        .process-button {
          background-color: var(--success-color, #4CAF50);
        }
        
        .process-button:hover {
          background-color: #45a049;
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
        <div class="button-group">
          <button class="process-button" id="processButton">Process Audio</button>
        </div>
        
        <audio id="audioPlayer" controls></audio>
        
        <div class="button-group">
          <button class="reset-button" id="resetButton">Reset to Original</button>
          <button class="download-button" id="downloadButton">Download Processed</button>
        </div>
      </div>
    `;

    this.audio = this.root.getElementById('audioPlayer') as HTMLAudioElement;
    this.processButton = this.root.getElementById('processButton') as HTMLButtonElement;
    this.resetButton = this.root.getElementById('resetButton') as HTMLButtonElement;
    this.downloadButton = this.root.getElementById('downloadButton') as HTMLButtonElement;
  }

  private setupEventListeners() {
    this.processButton.addEventListener('click', () => {
      this.dispatchEvent(new Event('processAudio', { bubbles: true, composed: true }));
    });

    this.resetButton.addEventListener('click', () => {
      this.dispatchEvent(new Event('resetAudio', { bubbles: true, composed: true }));
    });

    this.downloadButton.addEventListener('click', () => {
      this.dispatchEvent(new Event('downloadAudio', { bubbles: true, composed: true }));
    });
  }

  updateAudio(audioData: ArrayBuffer) {
    const blob = new Blob([audioData], { type: 'audio/wav' });
    const url = URL.createObjectURL(blob);
    
    // Clean up old URL
    if (this.audio.src && this.audio.src.startsWith('blob:')) {
      URL.revokeObjectURL(this.audio.src);
    }
    
    this.audio.src = url;
  }

  setEnabled(enabled: boolean) {
    this.processButton.disabled = !enabled;
    this.resetButton.disabled = !enabled;
    this.downloadButton.disabled = !enabled;
  }
}

customElements.define('audio-player', AudioPlayer);