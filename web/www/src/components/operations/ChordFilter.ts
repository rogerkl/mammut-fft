import { BaseOperation, OperationConfig } from './BaseOperation';

export class ChordFilterOperation extends BaseOperation {
  private noteFrequencies: Map<string, number> = new Map([
    ['C', 261.63], ['C#', 277.18], ['D', 293.66], ['D#', 311.13],
    ['E', 329.63], ['F', 349.23], ['F#', 369.99], ['G', 392.00],
    ['G#', 415.30], ['A', 440.00], ['A#', 466.16], ['B', 493.88]
  ]);

  protected getConfig(): OperationConfig {
    return {
      name: 'Chord Filter',
      description: 'Filter audio to keep only the frequencies of a chord and their harmonics.',
      controls: [
        // Basic frequency inputs
        ...Array.from({ length: 5 }, (_, i) => ([
          {
            type: 'number',
            id: `freq${i + 1}`,
            label: `Frequency ${i + 1}`,
            min: 0,
            max: 20000,
            step: 0.1,
            defaultValue: i === 0 ? 440 : 0,
            unit: 'Hz'
          },
          {
            type: 'slider',
            id: `amp${i + 1}`,
            label: `Amplitude ${i + 1}`,
            min: 0,
            max: 1,
            step: 0.01,
            defaultValue: i === 0 ? 1 : 0,
          }
        ])).flat(),
        {
          type: 'slider',
          id: 'width',
          label: 'Width',
          min: 0,
          max: 50,
          step: 1,
          defaultValue: 25,
          unit: 'cents',
          help: 'Width around each frequency to keep'
        },
        {
          type: 'slider',
          id: 'harmonics',
          label: 'Harmonic Strength',
          min: 0,
          max: 1,
          step: 0.01,
          defaultValue: 0.5,
          help: '0: Only fundamental, 1: Full harmonic series'
        }
      ]
    };
  }

  connectedCallback() {
    super.connectedCallback();
    this.addPianoKeyboard();
  }

  private addPianoKeyboard() {
    // Add custom piano keyboard UI after the controls
    const controlsContainer = this.root.querySelector('.controls-container');
    if (controlsContainer) {
      const pianoHTML = `
        <div class="piano-section">
          <h4>Quick Note Selection</h4>
          <div class="octave-selector">
            ${Array.from({ length: 8 }, (_, i) => 
              `<button class="octave-btn ${i === 4 ? 'active' : ''}" data-octave="${i}">${i}</button>`
            ).join('')}
          </div>
          <div class="piano-keyboard">
            ${this.renderPianoKeys()}
          </div>
        </div>
      `;
      
      controlsContainer.insertAdjacentHTML('afterend', pianoHTML);
      this.setupPianoEventListeners();
    }
  }

  private renderPianoKeys(): string {
    const whiteKeys = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
    // Each black key sits centered on the boundary *after* the white key at this index.
    // No black key after E (index 2) or B (index 6).
    const blackKeys: { note: string; afterWhiteIndex: number }[] = [
      { note: 'C#', afterWhiteIndex: 0 },
      { note: 'D#', afterWhiteIndex: 1 },
      { note: 'F#', afterWhiteIndex: 3 },
      { note: 'G#', afterWhiteIndex: 4 },
      { note: 'A#', afterWhiteIndex: 5 },
    ];
    const whiteWidthPct = 100 / whiteKeys.length;
    const blackWidthPct = whiteWidthPct * 0.6;

    return `
      <div class="white-keys">
        ${whiteKeys.map(note =>
          `<button class="key white-key" data-note="${note}">${note}</button>`
        ).join('')}
      </div>
      <div class="black-keys">
        ${blackKeys.map(({ note, afterWhiteIndex }) => {
          const boundary = (afterWhiteIndex + 1) * whiteWidthPct;
          const left = boundary - blackWidthPct / 2;
          return `<button class="key black-key" data-note="${note}" style="left:${left}%;width:${blackWidthPct}%">${note}</button>`;
        }).join('')}
      </div>
    `;
  }

  private setupPianoEventListeners() {
    // Octave selection
    let currentOctave = 4;
    this.root.querySelectorAll('.octave-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        this.root.querySelectorAll('.octave-btn').forEach(b => b.classList.remove('active'));
        (e.target as HTMLElement).classList.add('active');
        currentOctave = parseInt((e.target as HTMLElement).dataset.octave!);
      });
    });

    // Note selection
    this.root.querySelectorAll('.key').forEach(key => {
      key.addEventListener('click', (e) => {
        const note = (e.target as HTMLElement).dataset.note!;
        const freq = this.noteFrequencies.get(note)! * Math.pow(2, currentOctave - 4);
        this.addFrequencyToNextSlot(freq);
      });
    });
  }

  private addFrequencyToNextSlot(frequency: number) {
    // Find the first empty slot (freq = 0)
    for (let i = 1; i <= 5; i++) {
      const input = this.controls.get(`freq${i}`) as HTMLInputElement;
      if (input && (parseFloat(input.value) === 0 || input.value === '')) {
        input.value = frequency.toFixed(2);
        const ampInput = this.controls.get(`amp${i}`) as HTMLInputElement;
        if (ampInput) ampInput.value = '1';
        break;
      }
    }
  }

  protected onApply(values: Record<string, any>): void {
    const frequencies = [values.freq1, values.freq2, values.freq3, values.freq4, values.freq5];
    const amplitudes = [values.amp1, values.amp2, values.amp3, values.amp4, values.amp5];
    
    this.audioService!.applyChordFilter(
      frequencies,
      amplitudes,
      values.width,
      values.harmonics
    );
  }

  protected getStyles(): string {
    return super.getStyles() + `
      .piano-section {
        margin-top: 20px;
        padding: 15px;
        background-color: rgba(0, 0, 0, 0.15);
        border-radius: 8px;
      }
      
      .piano-section h4 {
        margin: 0 0 10px 0;
        color: var(--text-primary);
      }
      
      .octave-selector {
        display: flex;
        gap: 5px;
        margin-bottom: 10px;
      }
      
      .octave-btn {
        width: 30px;
        height: 30px;
        border: 1px solid var(--border-color);
        background: rgba(255, 255, 255, 0.1);
        color: var(--text-primary);
        cursor: pointer;
        border-radius: 4px;
      }
      
      .octave-btn.active {
        background: var(--accent-color);
        color: white;
      }
      
      .piano-keyboard {
        position: relative;
        height: 80px;
      }
      
      .white-keys {
        display: flex;
        position: absolute;
        width: 100%;
        height: 100%;
      }

      .black-keys {
        position: absolute;
        width: 100%;
        height: 60%;
      }

      .key {
        border: 1px solid #333;
        cursor: pointer;
        display: flex;
        align-items: flex-end;
        justify-content: center;
        padding-bottom: 5px;
        font-size: 12px;
        box-sizing: border-box;
      }

      .white-key {
        flex: 1;
        background: white;
        color: black;
      }

      .black-key {
        position: absolute;
        top: 0;
        height: 100%;
        background: #333;
        color: white;
        /* left and width set inline per key */
      }
      
      .key:active {
        background: var(--accent-color);
        color: white;
      }
    `;
  }
}

customElements.define('chord-filter-operation', ChordFilterOperation);