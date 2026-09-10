# Audio decode fixtures

Half-second 440 Hz sine tones, one per container/codec pair that
`audio_toolkit::decode` claims to support but that a synthetic WAV can never
exercise. They are committed rather than generated at test time so the
regression runs on any machine, with or without ffmpeg.

| File                      | Container | Codec  | Channels | Rate      | Why it is here                                                                                                                                                          |
| ------------------------- | --------- | ------ | -------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `aac-lc-mono-44100.m4a`   | MP4       | AAC-LC | 1        | 44 100 Hz | MP4 leaves `codec_params.channels` empty; the decoder must read the channel layout off the first decoded packet instead. Regression for the M4A rejection found in #45. |
| `vorbis-stereo-48000.ogg` | Ogg       | Vorbis | 2        | 48 000 Hz | The Ogg case that does work, so the Opus test below can't pass by rejecting every `.ogg`.                                                                               |
| `opus-mono-48000.ogg`     | Ogg       | Opus   | 1        | 48 000 Hz | symphonia 0.5 ships no Opus decoder. Must fail naming the codec, not with a generic "unsupported format".                                                               |

Regenerate with:

```bash
ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=44100" \
  -ac 1 -c:a aac -b:a 32k aac-lc-mono-44100.m4a
ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=48000" \
  -ac 2 -c:a vorbis -strict -2 vorbis-stereo-48000.ogg
ffmpeg -f lavfi -i "sine=frequency=440:duration=0.5:sample_rate=48000" \
  -ac 1 -c:a libopus -b:a 24k opus-mono-48000.ogg
```
