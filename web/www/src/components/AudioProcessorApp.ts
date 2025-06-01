import { AudioProcessorService } from '../services/AudioProcessor';

export class AudioProcessorApp extends HTMLElement {
  private audioService: AudioProcessorService;

  constructor() {
    super();
    this.audioService = new AudioProcessorService();
  }

  connectedCallback() {
    this.render();
    this.setupEventListeners();
  }

  disconnectedCallback() {
    this.audioService.destroy();
  }

  private render() {
    // Don't use shadow DOM for the main app component
    this.innerHTML = `
      <div class="container">
        <div class="header">
          <img src="/mammut-fft/mammut-fft-logo.png" alt="Mammut FFT" width="706" height="123">
        </div>
                
        <file-uploader></file-uploader>
        
        <status-bar></status-bar>        
        
        <div class="main-content" id="mainContent" style="display: none;">
          <audio-player></audio-player>
          <spectrum-visualizer></spectrum-visualizer>
          <operation-tabs></operation-tabs>
        </div>
      </div>
    `;
  }

  private setupEventListeners() {
    // Listen for file upload
    this.addEventListener('fileSelected', async (event: Event) => {
      const customEvent = event as CustomEvent<{ file: File; bufferMultiplier: number }>;
      const { file, bufferMultiplier } = customEvent.detail;
      
      try {
        this.updateStatus('Loading audio file...');
        await this.audioService.loadAudioFile(file, bufferMultiplier);
        
        // Show main content
        const mainContent = this.querySelector('#mainContent') as HTMLElement;
        if (mainContent) {
          mainContent.style.display = 'block';
        }
        
        // Enable audio player controls
        const audioPlayer = this.querySelector('audio-player') as any;
        audioPlayer?.setEnabled(true);
        
        // Update components
        this.updateStatus('Audio loaded successfully!', 'success');
        this.updateAllComponents();
      } catch (error) {
        this.updateStatus(`Error loading audio: ${error}`, 'error');
      }
    });

    // Listen for audio service events
    this.audioService.addEventListener('audioLoaded', async () => {
      this.updateAllComponents();
      const audioData = await this.audioService.processAudio();
      const audioPlayer = this.querySelector('audio-player') as any;
      audioPlayer?.updateAudio(audioData);
    });

    this.audioService.addEventListener('operationApplied', () => {
      this.updateStatus('Operation applied...');
      this.updateSpectrum();
      const audioPlayer = this.querySelector('audio-player') as any;
      audioPlayer?.resetProcessed();
    });

    this.audioService.addEventListener('reset', () => {
      this.updateAllComponents();
    });

    // Process audio button
    this.addEventListener('processAudio', async () => {
      try {
        this.updateStatus('Processing audio...');
        const audioData = await this.audioService.processAudio();
        
        // Update audio player
        const audioPlayer = this.querySelector('audio-player') as any;
        audioPlayer?.updateAudio(audioData);
        
        this.updateStatus('Audio processed successfully!', 'success');
      } catch (error) {
        this.updateStatus(`Error processing audio: ${error}`, 'error');
      }
    });

    // Download button
    this.addEventListener('downloadAudio', () => {
      try {
        this.audioService.downloadProcessedAudio();
        this.updateStatus('Audio downloaded successfully!', 'success');
      } catch (error) {
        this.updateStatus(`Error downloading audio: ${error}`, 'error');
      }
    });

    // Reset button
    this.addEventListener('resetAudio', async () => {
      this.audioService.reset();
      this.updateAllComponents();
      this.updateStatus('Audio reset to original state', 'success');
      const audioData = await this.audioService.processAudio();
      const audioPlayer = this.querySelector('audio-player') as any;
      audioPlayer?.updateAudio(audioData);
    });
  }

  private updateStatus(message: string, type: 'info' | 'success' | 'error' = 'info') {
    const statusBar = this.querySelector('status-bar') as any;
    statusBar?.setStatus(message, type);
  }

  private updateAllComponents() {
    this.updateSpectrum();
    this.updateOperations();
  }

  private updateSpectrum() {
    const visualizer = this.querySelector('spectrum-visualizer') as any;
    visualizer?.updateSpectrum(this.audioService);
  }

  private updateOperations() {
    const operations = this.querySelector('operation-tabs') as any;
    operations?.setAudioService(this.audioService);
  }
}

customElements.define('audio-processor-app', AudioProcessorApp);