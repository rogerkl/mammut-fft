import type { WasmAudioProcessor, AudioInfo } from '../types/mammut-fft';
import { WasmService } from './WasmService';

export class AudioProcessorService extends EventTarget {
  private processor: WasmAudioProcessor;
  private audioContext: AudioContext | null = null;
  private currentFileName: string = '';
  private hasUnprocessedOperations: boolean = false;

  constructor() {
    super();
    this.processor = WasmService.createProcessor();
  }

  async loadAudioFile(file: File, bufferMultiplier: number): Promise<void> {
      this.currentFileName = file.name;
      this.hasUnprocessedOperations = false; // Reset flag when loading new audio
      
      // Initialize audio context on user interaction
      if (!this.audioContext) {
        this.audioContext = new AudioContext();
      }
  
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
  
      try {
        this.processor.read_audio_bytes(uint8Array, bufferMultiplier);
        this.dispatchEvent(new CustomEvent('audioLoaded', { 
          detail: { fileName: file.name, info: this.getInfo() } 
        }));
      } catch (error) {
        throw new Error(`Failed to load audio: ${error}`);
      }
    }


  getInfo(): AudioInfo {
    return this.processor.get_info();
  }

  getSpectrumData(channelIndex: number, maxPoints: number, useLogScale: boolean): Float32Array {
    return this.processor.get_spectrum_data(channelIndex, maxPoints, useLogScale);
  }

  async processAudio(): Promise<ArrayBuffer> {
      this.processor.perform_ifft();
      this.hasUnprocessedOperations = false; // Reset flag since we just processed
      const processedData = this.processor.get_processed_audio_normalized();
      
      if (!this.audioContext) {
        throw new Error('Audio context not initialized');
      }
  
      const info = this.getInfo();
      const numChannels = info.channels;
      const length = processedData.length / numChannels;
  
      // Create AudioBuffer
      const audioBuffer = this.audioContext.createBuffer(
        numChannels,
        length,
        info.sample_rate
      );
  
      // Deinterleave and copy data
      for (let channel = 0; channel < numChannels; channel++) {
        const channelData = audioBuffer.getChannelData(channel);
        for (let i = 0; i < length; i++) {
          channelData[i] = processedData[i * numChannels + channel];
        }
      }
  
      // Convert to WAV
      return this.audioBufferToWav(audioBuffer);
    }


  downloadProcessedAudio(filename?: string): { wasProcessed: boolean } {
      let wasProcessed = false;
      
      // Check if there are unprocessed operations
      if (this.hasUnprocessedOperations) {
        // Automatically process the audio before download
        console.log('Audio operations detected - performing inverse FFT before download...');
        this.processor.perform_ifft();
        this.hasUnprocessedOperations = false;
        wasProcessed = true;
      }
  
      const wavData = this.processor.save_to_wav_bytes();
      const blob = new Blob([wavData], { type: 'audio/wav' });
      const url = URL.createObjectURL(blob);
  
      const outputFilename = filename || 
        (this.currentFileName ? 
          `${this.currentFileName.replace(/\.[^/.]+$/, '')}_processed.wav` : 
          'processed_audio.wav');
  
      const link = document.createElement('a');
      link.href = url;
      link.download = outputFilename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      
      URL.revokeObjectURL(url);
      
      return { wasProcessed };
    }



  reset(): void {
      this.processor.reset();
      this.hasUnprocessedOperations = false; // Reset flag when audio is reset
      this.dispatchEvent(new Event('reset'));
    }


  // Audio processing operations
  applyPower(exponent: number): void {
    this.processor.apply_pow(exponent);
    this.dispatchEvent(new Event('operationApplied'));
  }

  applyLowpass(cutoff: number): void {
    this.processor.apply_lowpass(cutoff);
    this.dispatchEvent(new Event('operationApplied'));
  }

  applyHighpass(cutoff: number): void {
    this.processor.apply_highpass(cutoff);
    this.dispatchEvent(new Event('operationApplied'));
  }

  applyBandpass(low: number, high: number): void {
    this.processor.apply_bandpass(low, high);
    this.dispatchEvent(new Event('operationApplied'));
  }

  // ... (other operations following the same pattern)

  private audioBufferToWav(buffer: AudioBuffer): ArrayBuffer {
    const numChannels = buffer.numberOfChannels;
    const length = buffer.length * numChannels * 2;
    const sampleRate = buffer.sampleRate;

    const arrayBuffer = new ArrayBuffer(44 + length);
    const view = new DataView(arrayBuffer);

    // WAV header
    const writeString = (offset: number, string: string) => {
      for (let i = 0; i < string.length; i++) {
        view.setUint8(offset + i, string.charCodeAt(i));
      }
    };

    writeString(0, 'RIFF');
    view.setUint32(4, 36 + length, true);
    writeString(8, 'WAVE');
    writeString(12, 'fmt ');
    view.setUint32(16, 16, true);
    view.setUint16(20, 1, true);
    view.setUint16(22, numChannels, true);
    view.setUint32(24, sampleRate, true);
    view.setUint32(28, sampleRate * numChannels * 2, true);
    view.setUint16(32, numChannels * 2, true);
    view.setUint16(34, 16, true);
    writeString(36, 'data');
    view.setUint32(40, length, true);

    // Interleave audio data
    let offset = 44;
    for (let i = 0; i < buffer.length; i++) {
      for (let channel = 0; channel < numChannels; channel++) {
        const sample = Math.max(-1, Math.min(1, buffer.getChannelData(channel)[i]));
        const int16 = sample < 0 ? sample * 0x8000 : sample * 0x7FFF;
        view.setInt16(offset, int16, true);
        offset += 2;
      }
    }

    return arrayBuffer;
  }

  destroy(): void {
    this.processor.free();
  }

  private dispatchOperationApplied()
  {
    this.hasUnprocessedOperations = true; // Mark that operations have been applied
    this.dispatchEvent(new Event('operationApplied'));
  }

  applyPhaseMultiply(factor: number): void {
    this.processor.apply_phase_multiply(factor);
  }

  applySpectrumShift(shiftHz: number): void {
    this.processor.apply_spectrum_shift(shiftHz);
    this.dispatchOperationApplied();
  }

  applyStretch(exponent: number): void {
    this.processor.apply_stretch(exponent);
    this.dispatchEvent(new Event('operationApplied'));
  }

  applyWobble(frequency: number, amplitude: number): void {
    this.processor.apply_wobble(frequency, amplitude);
    this.dispatchOperationApplied();
  }

  applyThreshold(level: number, removeAbove: boolean): void {
    this.processor.apply_threshold(level, removeAbove);
    this.dispatchOperationApplied();
  }

  applyAmplitudeDerivative(multiplier: number): void {
    this.processor.apply_amplitude_derivative(multiplier);
    this.dispatchOperationApplied();
  }

  keepPeaks(): void {
    this.processor.keep_peaks();
    this.dispatchOperationApplied();
  }

  swapBins(blockSize: number, repeat: number): void {
    this.processor.swap_bins(blockSize, repeat);
    this.dispatchOperationApplied();
  }

  swapChannels(repeat: number): void {
    this.processor.swap_channels(repeat);
    this.dispatchOperationApplied();
  }

  applyChordFilter(
    frequencies: number[],
    amplitudes: number[],
    widthCents: number,
    harmonicsStrength: number
  ): void {
    if (frequencies.length !== 5 || amplitudes.length !== 5) {
      throw new Error('Chord filter requires exactly 5 frequencies and amplitudes');
    }
    
    this.processor.apply_chord_filter(
      frequencies[0], amplitudes[0],
      frequencies[1], amplitudes[1],
      frequencies[2], amplitudes[2],
      frequencies[3], amplitudes[3],
      frequencies[4], amplitudes[4],
      widthCents,
      harmonicsStrength
    );
    this.dispatchOperationApplied();
  }

  prepareSplitPart(partIndex: number, numParts: number, groupSize: number, log: boolean): void {
    this.processor.prepare_split_part(partIndex, numParts, groupSize, log);
  }

  resetSplit(): void {
    this.processor.reset_split();
  }

  async convolveWithFile(audioData: Uint8Array, correlate: boolean, wetMix: number): Promise<void> {
    this.processor.convolve_with_file(audioData, correlate, wetMix);
    this.dispatchOperationApplied();
  }
}