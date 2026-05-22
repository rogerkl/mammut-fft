export interface AudioInfo {
  sample_rate: number;
  channels: number;
  fft_size: number;
  time_data?: TimeDataInfo;
  fft_data?: FFTDataInfo;
}

export interface TimeDataInfo {
  num_channels: number;
  samples_per_channel: number;
  duration_seconds: number;
}

export interface FFTDataInfo {
  num_channels: number;
  complex_values_per_channel: number;
  frequency_resolution: number;
}

export interface WasmAudioProcessor {
  new(): WasmAudioProcessor;
  free(): void;
  load_audio_data(channels: number, sample_rate: number, audio_data: Float32Array): void;
  load_audio_data_with_padding(channels: number, sample_rate: number, audio_data: Float32Array, buffer_multiplier: number): void;
  get_info(): AudioInfo;
  get_spectrum_data(channel_index: number, max_points: number, use_log_scale: boolean): Float32Array;
  apply_pow(exponent: number): void;
  apply_lowpass(cutoff_hz: number): void;
  apply_highpass(cutoff_hz: number): void;
  apply_bandpass(low_hz: number, high_hz: number): void;
  apply_phase_shift(shift_radians: number): void;
  apply_phase_multiply(factor: number): void;
  apply_spectrum_shift(shift_hz: number): void;
  apply_stretch(exponent: number): void;
  apply_wobble(frequency: number, amplitude: number): void;
  apply_threshold(threshold_level: number, remove_above: boolean): void;
  apply_amplitude_derivative(multiplier: number): void;
  apply_chord_filter(
    freq1: number, amp1: number,
    freq2: number, amp2: number,
    freq3: number, amp3: number,
    freq4: number, amp4: number,
    freq5: number, amp5: number,
    width_cents: number,
    harmonics_strength: number
  ): void;
  keep_peaks(): void;
  swap_bins(block_size: number, repeat: number): void;
  swap_channels(repeat: number): void;
  perform_ifft(): void;
  normalize_time_data(): void;
  get_processed_audio_normalized(): Float32Array;
  save_to_wav_bytes(): Uint8Array;
  read_audio_bytes(data: Uint8Array, buffer_multiplier: number): void;
  reset(): void;
  reset_split(): void;
  reset_to_original(): void;
  prepare_split_part(part_index: number, num_parts: number, group_size: number,log: boolean, octaves: number, crossfade_factor: number): void;
  mix_channels(weights: Float64Array): void;
  convolve_with_file(audio_data: Uint8Array, correlate: boolean, wet_mix: number): void;
}

export interface WasmModule {
  WasmAudioProcessor: typeof WasmAudioProcessor;
}