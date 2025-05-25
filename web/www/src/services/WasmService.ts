import type { WasmModule, WasmAudioProcessor } from '../types/mammut-fft';

export class WasmService {
  private static wasmModule: WasmModule | null = null;
  private static initPromise: Promise<void> | null = null;

  static async initialize(): Promise<void> {
    if (this.initPromise) return this.initPromise;

    this.initPromise = this.loadWasm();
    await this.initPromise;
  }

  private static async loadWasm(): Promise<void> {
    // Use a relative path that Vite can resolve
    const { default: init, WasmAudioProcessor } = await import('../../pkg/mammut_fft_web.js');
    await init();
    
    this.wasmModule = { WasmAudioProcessor };
  }

  static createProcessor(): WasmAudioProcessor {
    if (!this.wasmModule) {
      throw new Error('WASM module not initialized');
    }
    return new this.wasmModule.WasmAudioProcessor();
  }
}