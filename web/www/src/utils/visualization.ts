import type { AudioInfo } from '../types/mammut-fft';

export function drawSpectrum(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  spectrumData: Float32Array,
  info: AudioInfo
) {
  // Clear canvas
  ctx.fillStyle = '#2a2a2a';
  ctx.fillRect(0, 0, width, height);

  if (!spectrumData || spectrumData.length === 0) {
    drawEmptyMessage(ctx, width, height, 'No spectrum data available');
    return;
  }

  // Draw grid
  drawGrid(ctx, width, height);
  
  // Draw amplitude scale
  drawAmplitudeScale(ctx, height);
  
  // Draw frequency labels
  drawLogFrequencyLabels(ctx, width, height, info);
  
  // Smooth the data
  const smoothedData = smoothData(spectrumData, 3);
  
  // Create gradient
  const gradient = ctx.createLinearGradient(0, height, 0, 0);
  gradient.addColorStop(0, 'rgba(37, 134, 255, 0.7)');
  gradient.addColorStop(0.5, 'rgba(92, 195, 255, 0.8)');
  gradient.addColorStop(1, 'rgba(255, 255, 255, 0.9)');
  
  // Draw spectrum
  ctx.beginPath();
  ctx.moveTo(0, height);
  
  const barWidth = width / smoothedData.length;
  for (let i = 0; i < smoothedData.length; i++) {
    const x = i * barWidth;
    const y = height - smoothedData[i] * height;
    ctx.lineTo(x, y);
  }
  
  ctx.lineTo(width, height);
  ctx.closePath();
  
  ctx.fillStyle = gradient;
  ctx.fill();
  
  // Draw line on top
  ctx.beginPath();
  ctx.moveTo(0, height);
  for (let i = 0; i < smoothedData.length; i++) {
    const x = i * barWidth;
    const y = height - smoothedData[i] * height;
    ctx.lineTo(x, y);
  }
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
  ctx.lineWidth = 2;
  ctx.stroke();
  
  // Draw peak indicators
  drawPeakIndicators(ctx, width, height, smoothedData, info);
}

export function drawEmptySpectrum(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number
) {
  ctx.fillStyle = '#2a2a2a';
  ctx.fillRect(0, 0, width, height);
  
  drawGrid(ctx, width, height);
  drawAmplitudeScale(ctx, height);
  drawLogFrequencyLabels(ctx, width, height, { sample_rate: 44100 } as AudioInfo);
  drawEmptyMessage(ctx, width, height, 'Load an audio file to see the spectrum');
}

function drawEmptyMessage(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  message: string
) {
  ctx.fillStyle = '#aaaaaa';
  ctx.font = '16px Arial';
  ctx.textAlign = 'center';
  ctx.fillText(message, width / 2, height / 2);
}

function drawGrid(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number
) {
  ctx.strokeStyle = 'rgba(100, 100, 100, 0.2)';
  ctx.lineWidth = 1;
  
  // Horizontal grid lines
  for (let i = 0; i < 10; i++) {
    const y = i * (height / 10);
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }
  
  // Vertical grid lines (logarithmic)
  const frequencies = [20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000];
  for (const freq of frequencies) {
    if (freq >= 20 && freq <= 20000) {
      const x = logFreqToX(freq, width);
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.strokeStyle = freq % 1000 === 0 ? 
        'rgba(150, 150, 150, 0.3)' : 
        'rgba(100, 100, 100, 0.2)';
      ctx.stroke();
    }
  }
}

function drawAmplitudeScale(ctx: CanvasRenderingContext2D, height: number) {
  ctx.fillStyle = '#aaaaaa';
  ctx.font = '12px Arial';
  ctx.textAlign = 'left';
  
  const levels = [0, 0.25, 0.5, 0.75, 1.0];
  for (const level of levels) {
    const y = height - level * height;
    const label = Math.round(level * 100) + '%';
    
    ctx.fillRect(0, y, 4, 1);
    ctx.fillText(label, 7, y + 4);
  }
}

function drawLogFrequencyLabels(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  info: AudioInfo
) {
  if (!info.sample_rate) return;
  
  ctx.fillStyle = '#aaaaaa';
  ctx.font = '12px Arial';
  ctx.textAlign = 'center';
  
  const freqLabels = [20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000];
  
  for (const freq of freqLabels) {
    if (freq >= 20 && freq <= 20000) {
      const x = logFreqToX(freq, width);
      
      let label: string;
      if (freq >= 1000) {
        label = `${freq / 1000}kHz`;
      } else {
        label = `${freq}Hz`;
      }
      
      // Only show some labels to avoid crowding
      if ([20, 100, 1000, 10000, 20000].includes(freq)) {
        ctx.fillText(label, x, height - 5);
      }
    }
  }
}

function drawPeakIndicators(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  data: Float32Array,
  info: AudioInfo
) {
  if (!info.sample_rate) return;
  
  const peaks = findPeaks(data, 0.5, Math.floor(data.length / 50));
  if (peaks.length === 0) return;
  
  const nyquistFreq = info.sample_rate / 2;
  
  ctx.fillStyle = 'rgba(255, 220, 100, 0.9)';
  ctx.font = '10px Arial';
  ctx.textAlign = 'center';
  
  for (const peakIndex of peaks) {
    const x = peakIndex * (width / data.length);
    const y = height - data[peakIndex] * height;
    
    // Draw peak marker
    ctx.beginPath();
    ctx.arc(x, y, 3, 0, Math.PI * 2);
    ctx.fill();
    
    // Calculate frequency
    const normalizedPosition = peakIndex / data.length;
    const minFreq = 20;
    const maxFreq = nyquistFreq;
    const minLog = Math.log10(minFreq);
    const maxLog = Math.log10(maxFreq);
    const logFreq = minLog + normalizedPosition * (maxLog - minLog);
    const freq = Math.pow(10, logFreq);
    
    // Format frequency label
    let freqLabel: string;
    if (freq >= 1000) {
      freqLabel = `${(freq / 1000).toFixed(1)}kHz`;
    } else {
      freqLabel = `${Math.round(freq)}Hz`;
    }
    
    ctx.fillText(freqLabel, x, y - 10);
  }
}

function logFreqToX(freq: number, width: number): number {
  const minFreq = 20;
  const maxFreq = 20000;
  const minLog = Math.log10(minFreq);
  const maxLog = Math.log10(maxFreq);
  const logRange = maxLog - minLog;
  const logFreq = Math.log10(freq);
  const normalizedPos = (logFreq - minLog) / logRange;
  return normalizedPos * width;
}

function smoothData(data: Float32Array, windowSize: number): Float32Array {
  if (windowSize <= 1 || data.length <= windowSize) {
    return data;
  }
  
  const result = new Float32Array(data.length);
  
  // Handle edges
  for (let i = 0; i < windowSize - 1; i++) {
    result[i] = data[i];
  }
  
  // Apply moving average
  for (let i = windowSize - 1; i < data.length; i++) {
    let sum = 0;
    for (let j = 0; j < windowSize; j++) {
      sum += data[i - j];
    }
    result[i] = sum / windowSize;
  }
  
  return result;
}

function findPeaks(
  data: Float32Array,
  threshold: number,
  minDistance: number
): number[] {
  const peaks: number[] = [];
  const minAmp = threshold;
  
  for (let i = 1; i < data.length - 1; i++) {
    if (data[i] > data[i - 1] && 
        data[i] > data[i + 1] && 
        data[i] >= minAmp) {
      
      const farEnough = peaks.every(
        peakIndex => Math.abs(i - peakIndex) >= minDistance
      );
      
      if (farEnough) {
        peaks.push(i);
        if (peaks.length >= 5) break;
      }
    }
  }
  
  return peaks;
}