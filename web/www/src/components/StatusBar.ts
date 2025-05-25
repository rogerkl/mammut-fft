export class StatusBar extends HTMLElement {
  private shadowRoot: ShadowRoot;
  private statusElement: HTMLDivElement;
  private timeoutId: number | null = null;

  constructor() {
    super();
    this.shadowRoot = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
  }

  private render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          margin: 20px 0;
        }
        
        .status {
          background-color: rgba(255, 255, 255, 0.05);
          border-left: 4px solid var(--accent-color, #4f9eff);
          padding: 12px 18px;
          font-size: 14px;
          color: var(--text-primary, #e0e0e0);
          border-radius: 0 4px 4px 0;
          transition: all 0.3s ease;
          min-height: 20px;
        }
        
        .status.error {
          background-color: var(--error-bg, rgba(244, 67, 54, 0.1));
          border-left-color: var(--error-color, #f44336);
          color: var(--error-color, #f44336);
        }
        
        .status.success {
          background-color: var(--success-bg, rgba(76, 175, 80, 0.1));
          border-left-color: var(--success-color, #4CAF50);
          color: var(--success-color, #4CAF50);
        }
        
        .status.hidden {
          opacity: 0;
          transform: translateY(-10px);
        }
      </style>
      
      <div class="status" id="statusElement">
        Ready to load audio file. Choose or drop a WAV file to begin.
      </div>
    `;

    this.statusElement = this.shadowRoot.getElementById('statusElement') as HTMLDivElement;
  }

  setStatus(message: string, type: 'info' | 'success' | 'error' = 'info', autoHide: boolean = false) {
    // Clear any existing timeout
    if (this.timeoutId) {
      clearTimeout(this.timeoutId);
      this.timeoutId = null;
    }

    // Update status
    this.statusElement.textContent = message;
    this.statusElement.className = 'status';
    
    if (type === 'error') {
      this.statusElement.classList.add('error');
    } else if (type === 'success') {
      this.statusElement.classList.add('success');
    }

    // Remove hidden class if it exists
    this.statusElement.classList.remove('hidden');

    // Auto-hide success messages after 5 seconds
    if (autoHide || type === 'success') {
      this.timeoutId = window.setTimeout(() => {
        this.statusElement.classList.add('hidden');
      }, 5000);
    }
  }
}

customElements.define('status-bar', StatusBar);