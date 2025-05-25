export class FileUploader extends HTMLElement {
  private shadowRoot: ShadowRoot;
  private fileInput: HTMLInputElement;
  private bufferMultiplierInput: HTMLInputElement;

  constructor() {
    super();
    this.shadowRoot = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    this.setupEventListeners();
  }

  private render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          margin: 20px 0;
        }
        
        .upload-section {
          border: 2px dashed var(--border-color, #333);
          border-radius: 8px;
          padding: 20px;
          text-align: center;
          transition: all 0.3s ease;
          background-color: rgba(255, 255, 255, 0.02);
        }
        
        .upload-section:hover {
          border-color: var(--accent-color, #4f9eff);
          background-color: rgba(255, 255, 255, 0.04);
        }
        
        .upload-section.drag-over {
          background-color: rgba(79, 158, 255, 0.1);
          border-color: var(--accent-color, #4f9eff);
          box-shadow: 0 0 10px rgba(79, 158, 255, 0.2);
        }
        
        .file-input-wrapper {
          display: flex;
          justify-content: center;
          align-items: center;
          gap: 20px;
          margin-top: 15px;
        }
        
        input[type="file"] {
          display: none;
        }
        
        .upload-button {
          padding: 12px 24px;
          background-color: var(--accent-color, #4f9eff);
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 15px;
          font-weight: 500;
          transition: background-color 0.2s;
        }
        
        .upload-button:hover {
          background-color: var(--accent-hover, #3d7dcf);
        }
        
        .buffer-control {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        
        .buffer-control label {
          font-size: 14px;
          color: var(--text-secondary, #aaa);
        }
        
        .buffer-control input {
          width: 60px;
          padding: 6px;
          background-color: rgba(0, 0, 0, 0.2);
          border: 1px solid var(--border-color, #333);
          border-radius: 4px;
          color: var(--text-primary, #e0e0e0);
        }
        
        .file-info {
          margin-top: 15px;
          font-size: 14px;
          color: var(--text-secondary, #aaa);
        }
        
        .drag-message {
          display: none;
          margin: 15px 0;
          font-size: 16px;
          color: var(--accent-color, #4f9eff);
          font-weight: 500;
        }
        
        .upload-section.drag-over .drag-message {
          display: block;
        }
      </style>
      
      <div class="upload-section" id="uploadSection">
        <label>Select audio file to process or drop file here</label>
        <div class="file-input-wrapper">
          <input type="file" id="audioFile" accept="audio/*">
          <button class="upload-button" id="uploadButton">Open Audio</button>
          
          <div class="buffer-control">
            <label for="bufferMultiplier">Buffer Size:</label>
            <input type="number" id="bufferMultiplier" min="1" max="8" value="1">
          </div>
        </div>
        <div class="drag-message">Release to upload audio file</div>
        <div class="file-info" id="fileInfo"></div>
      </div>
    `;

    this.fileInput = this.shadowRoot.getElementById('audioFile') as HTMLInputElement;
    this.bufferMultiplierInput = this.shadowRoot.getElementById('bufferMultiplier') as HTMLInputElement;
  }

  private setupEventListeners() {
    const uploadSection = this.shadowRoot.getElementById('uploadSection')!;
    const uploadButton = this.shadowRoot.getElementById('uploadButton')!;

    // Click to upload
    uploadButton.addEventListener('click', () => {
      this.fileInput.click();
    });

    // File input change
    this.fileInput.addEventListener('change', (e) => {
      const target = e.target as HTMLInputElement;
      if (target.files?.length) {
        this.handleFile(target.files[0]);
      }
    });

    // Drag and drop
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
      uploadSection.addEventListener(eventName, this.preventDefaults);
    });

    ['dragenter', 'dragover'].forEach(eventName => {
      uploadSection.addEventListener(eventName, () => {
        uploadSection.classList.add('drag-over');
      });
    });

    ['dragleave', 'drop'].forEach(eventName => {
      uploadSection.addEventListener(eventName, () => {
        uploadSection.classList.remove('drag-over');
      });
    });

    uploadSection.addEventListener('drop', (e) => {
      const dt = (e as DragEvent).dataTransfer;
      if (dt?.files.length) {
        this.handleFile(dt.files[0]);
      }
    });
  }

  private preventDefaults(e: Event) {
    e.preventDefault();
    e.stopPropagation();
  }

  private handleFile(file: File) {
    const fileInfo = this.shadowRoot.getElementById('fileInfo')!;
    const fileSize = (file.size / 1024).toFixed(1);
    fileInfo.textContent = `Selected: ${file.name} (${fileSize} KB)`;

    const bufferMultiplier = parseInt(this.bufferMultiplierInput.value);

    // Dispatch event to parent
    this.dispatchEvent(new CustomEvent('fileSelected', {
      detail: { file, bufferMultiplier },
      bubbles: true,
      composed: true
    }));
  }
}

customElements.define('file-uploader', FileUploader);