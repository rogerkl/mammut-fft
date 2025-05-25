import { AudioProcessorService } from '../services/AudioProcessor';
import { drawSpectrum, drawEmptySpectrum } from '../utils/visualization';

export class SpectrumVisualizer extends HTMLElement {
  private shadowRoot: ShadowRoot;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private audioService: AudioProcessorService | null = null;

  constructor() {
    super();
    this.shadowRoot = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    this.setupCanvas();
    drawEmptySpectrum(this.ctx, this.canvas.width, this.canvas.height);
  }

  private render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          margin: 25px 0;
        }
        
        canvas {
          width: 100%;
          height: 300px;
          background-color: var(--canvas-bg, #2a2a2a);
          border-radius: 8px;
          box-shadow: inset 0 0 10px rgba(0, 0, 0, 0.2);
        }
      </style>
      
      <canvas id="spectrumCanvas" width="800" height="300"></canvas>
    `;
  }

  private setupCanvas() {
    this.canvas = this.shadowRoot.getElementById('spectrumCanvas') as HTMLCanvasElement;
    this.ctx = this.canvas.getContext('2d')!;
    
    // Handle resize
    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        const { width } = entry.contentRect;
        this.canvas.width = width;
        this.updateSpectrum(this.audioService);
      }
    });
    
    resizeObserver.observe(this.canvas);
  }

  updateSpectrum(audioService: AudioProcessorService | null) {
    this.audioService = audioService;
    
    if (!audioService) {
      drawEmptySpectrum(this.ctx, this.canvas.width, this.canvas.height);
      return;
    }

    try {
      const info = audioService.getInfo();
      const maxPoints = this.canvas.width;
      const spectrumData = audioService.getSpectrumData(0, maxPoints, true);
      
      drawSpectrum(this.ctx, this.canvas.width, this.canvas.height, spectrumData, info);
    } catch (error) {
      console.error('Error drawing spectrum:', error);
      drawEmptySpectrum(this.ctx, this.canvas.width, this.canvas.height);
    }
  }
}

customElements.define('spectrum-visualizer', SpectrumVisualizer);