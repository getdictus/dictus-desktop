# Recording sounds: provenance and licence

Each pack is a folder with `start.wav` (played when recording starts) and `end.wav` (played when the text is inserted). Issue: getdictus/dictus-desktop#73.

## Bois, Synthé, Minimal

- Generated on 2026-11-19 with ElevenLabs Sound Effects, model `eleven_text_to_sound_v2`, through the `elevenlabs text-to-sound-effects convert` CLI on the maintainer's account.
- Licence: commercial use permitted by the account's plan, confirmed by the maintainer on 2026-11-19.
- Common parameters: `prompt_influence=0.7`, `loop=false`, `output_format=mp3_44100_192`. Three requests per prompt (v1, v2, v3); the maintainer listened to all of them and picked one per sound.
- Post-processing (ffmpeg): leading silence trimmed (threshold -50 dB), cut to the target length, 80 ms linear fade-out, peak normalised to -14 dBFS, WAV 16-bit 48 kHz stereo.
- Second pass on 2026-11-24, `scripts/feedback-sounds.py process` on the files above (commit `880a552`). The trim had cut into the attack of Bois and Synthé, which started at about 0.2 in their first millisecond: a click, heard as crackle on laptop speakers. Per file: 2-pole high-pass at 130 Hz, 6 ms raised-cosine fade-in, then one gain. Bois and Synthé are matched at -28.1 LUFS (BS.1770, ungated, over the whole sound); Minimal cannot reach it within the peak limit, so its two files share -37.3 LUFS. Peaks at or under -16 dBFS. Length, sample rate and pitch unchanged. `scripts/feedback-sounds.py measure` gives the numbers.

| File                | Variant          | Requested `duration_seconds` | Cut to |
| ------------------- | ---------------- | ---------------------------- | ------ |
| `bois/start.wav`    | bois-start v2    | 1.0                          | 0.45 s |
| `bois/end.wav`      | bois-end v1      | 1.2                          | 0.60 s |
| `synthe/start.wav`  | synthe-start v2  | 1.0                          | 0.45 s |
| `synthe/end.wav`    | synthe-end v3    | 1.2                          | 0.60 s |
| `minimal/start.wav` | minimal-start v1 | 0.8                          | 0.35 s |
| `minimal/end.wav`   | minimal-end v2   | 0.8                          | 0.45 s |

Prompts, verbatim:

- **Bois, start.** Short soft UI notification sound: a gentle wooden mallet tapping a small tuned wooden block, warm round ringing tone, bright and slightly high pitch, soft attack, then a quick rhythmic echo of three to four repeats about 130 milliseconds apart that fade out fast. Clean, dry, isolated single sound, no music, no voice, no ambience.
- **Bois, end.** Short soft UI notification sound: a gentle wooden mallet tapping a small tuned wooden block, warm round ringing tone, lower pitch, darker and more muffled, soft attack, then a quick rhythmic echo of three to four repeats about 130 milliseconds apart that fade out fast. Clean, dry, isolated single sound, no music, no voice, no ambience.
- **Synthé, start.** Short soft UI notification sound: a warm analog synthesizer blip, round sine-like tone through a gentle low-pass filter, bright high note, soft attack, then a gentle rhythmic delay of three to four repeats about 130 milliseconds apart that fade out fast. Clean, isolated single sound, no music, no melody, no voice, no ambience.
- **Synthé, end.** Short soft UI notification sound: a warm analog synthesizer blip, round sine-like tone with the low-pass filter more closed, lower note, darker and more muffled, soft attack, then a gentle rhythmic delay of three to four repeats about 130 milliseconds apart that fade out fast. Clean, isolated single sound, no music, no melody, no voice, no ambience.
- **Minimal, start.** Very short, very soft, discreet UI click: a tiny light tick, crisp but gentle, slightly bright, then a tiny faint echo of two quick repeats that vanish immediately. Clean, dry, isolated single sound, no music, no voice, no ambience.
- **Minimal, end.** Very short, very soft, discreet UI click: a tiny light tick, gentle, slightly lower and more muffled, then a tiny faint echo of two quick repeats that vanish immediately. Clean, dry, isolated single sound, no music, no voice, no ambience.

## Marimba, Pop

Inherited from Handy (cjpais/Handy, MIT). Marimba was added by CJ Pais on 2025-06-26 (Handy commit `528bfc2`, then renamed from `rec_start.wav`/`rec_stop.wav` in `a286fe5`). Pop was added in Handy PR #214 (commit `a286fe5`, 2025-10-17) and trimmed in Handy PR #349. Neither commit records where the audio came from or under which licence. **Origin and licence unknown.** They stay for existing installs that selected them and are not offered in the main picker.
