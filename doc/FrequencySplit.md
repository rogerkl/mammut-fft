# Frequency Split

The **Spectrum Split** operation takes a single audio file, splits its frequency spectrum across several output files, and saves each one. Every output file contains only a subset of the original bins — everything else is zeroed before the inverse FFT — so the parts together cover the whole spectrum, and summing them back recreates the original signal.

This page explains each UI control: how it shapes which bin goes to which part, and what it sounds like.

Implementation: `lib/src/operations/split.rs` and `lib/src/operations/crossfade.rs`.

## Background: bins and parts

After the full-file real FFT, a signal of length `N` becomes `N/2 + 1` complex frequency bins:

- bin `0` — DC (the constant component)
- bin `k` — frequency `k * sample_rate / N` Hz
- bin `N/2` — the Nyquist frequency (`sample_rate / 2`)

A *part* is one of the output files. Each part owns a set of bins. Other bins are set to 0 in that part's spectrum, then the inverse FFT is run to produce its audio.

The five controls below all decide one thing: **which bins go to which part**.

## Number of Parts

How many output files are produced. They are written as `<base>_0.wav`, `<base>_1.wav`, …

Part 0 is special:

- DC (bin 0) is always assigned to part 0.
- When the *Octaves to process* limit hides the lowest bins, those bins are also dumped into part 0.

## Group Size

Controls the **chunk size** of contiguous bin runs assigned to the same part.

- **`0` — band split.** The spectrum is divided into `num_parts` contiguous chunks. Part 0 owns the lowest band, part 1 the next, and so on. One frequency band per part.
- **`≥ 1` — interleaved split.** The spectrum is first divided into many thin chunks of `group_size` bins each. Those chunks are then handed to parts in round-robin order: chunk `i` goes to part `i mod num_parts`. Each part ends up holding chunks scattered all across the spectrum.

Example from the source docs (16 bins, group_size 2, num_parts 4):

```
bin[ 0] = 0   (DC, always part 0)
bin[ 1] = 0
bin[ 2] = 0
bin[ 3] = 1   group of 2 done — next group
bin[ 4] = 1
bin[ 5] = 2
bin[ 6] = 2
bin[ 7] = 3
bin[ 8] = 3
bin[ 9] = 0   parts exhausted — wrap back to 0
bin[10] = 0
bin[11] = 1
...
```

Rough heuristics:

| Effect | Settings |
|---|---|
| Each part is one band of the spectrum | `group_size = 0` |
| Each part is a "comb" picking out narrow lines | `group_size = 1`, `num_parts ≥ 2` |
| Wider-toothed comb | `group_size = 4..16` |

> ⚠️ The UI help text currently says `0 = num bins / num parts`. That's misleading — `0` is the *band split*, not an auto-pick. (See "Known UX issues" below.)

## Use log distribution

Switches the mapping from bins to parts.

- **off (linear).** Equal *bin count* per part. Since bins are uniformly spaced in Hz, this is also equal *Hz* per part. Musically lopsided: part 0 covers roughly the bottom octave plus everything *between* the second octave and ~5 kHz, while part 1 covers ~5–11 kHz, etc.
- **on (log).** Equal *log-frequency* per part. The frequency of each bin is `ln`-transformed, normalised into the log range, and the part is assigned by `floor(normalised * num_parts)`. Each part covers the same *ratio* of frequencies, so adjacent parts span the same number of musical octaves.

If you want a bass / mid / treble split that sounds balanced, use log distribution.

When **Group Size > 0**, log distribution is applied at the fine chunk level *first*, then the chunks are interleaved across parts by `mod num_parts`. Each part still picks an even, musically distributed selection of chunks.

## Octaves to process *(log distribution only)*

Restricts how much of the spectrum the log distribution actually covers, counting **down from Nyquist**.

The starting bin is:

```
start_bin = 1 + num_freq_bins / 2^octaves
```

So `octaves` is how many octaves *below* Nyquist the log split starts. Some sample values for a 44.1 kHz / Nyquist 22.05 kHz signal:

| Octaves | Lowest frequency in the split | What happens below |
|---:|---|---|
| 1 | ~11 kHz | Everything below 11 kHz → part 0 |
| 5 | ~700 Hz | Everything below 700 Hz → part 0 |
| 10 | ~22 Hz | Just sub-audible bins → part 0 |
| 12 | ~5 Hz | Effectively the full audible range is split |

Why this exists: with full log distribution, the first "octave" mathematically covers bins 1–N/2 — but bin 1 is around 1 Hz or less, and `ln` exaggerates that empty bottom end. Capping at a sensible number of octaves stops the lowest part from being mostly silence and gives you control over where the split "starts".

This control has **no effect** when *Use log distribution* is off.

## Crossfade between parts

Without crossfade, the boundary between parts is *hard*: bin `K` is 100% in part A, bin `K+1` is 100% in part B. That sharp transition is audible — when you remix the parts you can hear ringing at the boundary, and the individual parts sound like aggressively band-passed filters.

**Crossfade** softens each boundary by letting bins near a part edge belong to **both** sides with a linear amplitude weight. The shared bins are weighted so that they sum back to `1.0` when the parts are recombined, so total energy is preserved.

Behaviour:

- `0` — hard cut (default). Existing behaviour, no crossfade.
- `0.5` — each part borrows up to half its bin count from each neighbour. Maximum smoothing.

The math, for crossfade factor `f` at the boundary where part A ends and part B begins:

- Last `bin_count(A) * f` bins of A: amplitude `1.0 → 0.5` (linear fade-out).
- Same bins, also contributed to B: amplitude `0.0 → 0.5` (fade-in from the previous side).
- First `bin_count(B) * f` bins of B: amplitude `0.5 → 1.0` (continuing the fade-in).
- Same bins, also kept in A: amplitude `0.5 → 0.0` (fade-out into the next side).

At any shared bin the two contributions sum to `1.0`, so the total spectrum is preserved.

Implementation detail: each part's fade regions are represented as up to four `BinBoundaries` blocks (`fade_out_start`, `fade_in_start`, `fade_out_end`, `fade_in_end`) on a `FadeBoundariesPart` value, and `bin_amplitudes()` flattens them into the final `(bin, amplitude)` list used during the split. See `lib/src/operations/crossfade.rs`.

## Combining the options

The five controls compose like this:

```
num_parts        →  how many output files
group_size       →  band split (0) or interleaved chunks (≥1)
log distribution →  linear-Hz or log-Hz spacing of those chunks
octaves          →  on log only: cap where the split starts
crossfade        →  smooth boundaries instead of hard cuts
```

## Cookbook

| Goal | num_parts | group_size | log | octaves | crossfade |
|---|---:|---:|:---:|---:|---:|
| Low / mid / high split | 3 | 0 | off | — | 0 |
| Musical bass / mid / treble | 3 | 0 | on | 10 | 0 |
| One file per octave | `N` | 0 | on | `N` | 0 |
| Two-file comb filter (alt bins) | 2 | 1 | off | — | 0 |
| Wider comb | 2 | 8 | off | — | 0 |
| Any of the above, smoother | … | … | … | … | 0.1 – 0.3 |

If a split sounds "buzzy" at the boundaries, raise crossfade. If part 0 is dominating in a log split, raise *Octaves to process* (cap less aggressively).

## Known UX issues

- The *Octaves to process* slider has no visible effect when *Use log distribution* is off; it could be hidden or greyed out in that state.

## CLI equivalent

The CLI exposes the same operation via:

```
split <filename> <num_parts> [group_size] [log|lin] [octaves] [crossfade_factor]
```

The defaults are slightly different from the web UI: the CLI defaults to `group_size = 0` (band split), the web UI defaults to `group_size = 1`. `crossfade_factor` must be in `[0.0, 0.5]`. See `cli/src/main.rs` for the parser.
