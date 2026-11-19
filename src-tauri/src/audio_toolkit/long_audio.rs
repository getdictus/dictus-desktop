//! Splitting long recordings into windows an engine transcribes well.
//!
//! Some engines degrade or fail when one call gets a long recording: Parakeet V3
//! drifts into English on long French takes (#82), Moonshine rejects anything over
//! 64 s (#85). [`plan_windows`] cuts a recording into consecutive windows no longer
//! than the engine's limit, each cut placed in the longest pause near the end of the
//! window, so a cut rarely falls inside a word and tends to land between sentences.
//! [`join_texts`] stitches the per-window texts back together.
//!
//! Windows do not overlap, so no word is heard twice and the joins need no
//! de-duplication. A recording that fits in one window gives exactly one window.

use std::ops::Range;

use super::constants::WHISPER_SAMPLE_RATE;

const SAMPLE_RATE: f32 = WHISPER_SAMPLE_RATE as f32;

/// 30 ms at 16 kHz, the recorder's VAD frame.
const FRAME: usize = 480;

/// A frame is quiet below this fraction of the recording's loud level (−20 dB).
const QUIET_RATIO: f32 = 0.1;

/// A frame is always quiet below −60 dBFS RMS, whatever the recording's level.
const QUIET_FLOOR: f32 = 0.001;

/// A window with no frame above −55 dBFS RMS holds no speech worth an engine call.
const SPEECH_FLOOR: f32 = 0.0018;

/// Split `samples` (16 kHz mono) into consecutive windows of at most `max_secs`.
///
/// The windows cover every sample once, in order. A recording of at most `max_secs`
/// gives the single window `0..len`. Otherwise each cut is searched in the second half
/// of the window: the longest run of quiet frames wins (the later one on a tie) and the
/// cut goes to its middle; with no quiet frame, the cut goes to the quietest frame. The
/// last window is kept at least a quarter of `max_secs` long.
pub fn plan_windows(samples: &[f32], max_secs: f32) -> Vec<Range<usize>> {
    let len = samples.len();
    let max = ((max_secs * SAMPLE_RATE) as usize).max(2 * FRAME);
    let mut windows = Vec::new();
    let mut start = 0;
    if len > max {
        let rms: Vec<f32> = samples.chunks(FRAME).map(frame_rms).collect();
        let threshold = quiet_threshold(&rms);
        let quiet: Vec<bool> = rms.iter().map(|&r| r < threshold).collect();
        let min_tail = max / 4;
        while len - start > max {
            let lo = start + max / 2;
            let hi = (start + max).min(len - min_tail);
            let cut = best_cut(&rms, &quiet, lo, hi);
            windows.push(start..cut);
            start = cut;
        }
    }
    windows.push(start..len);
    windows
}

/// True when no 30 ms frame of `samples` reaches speech level.
pub fn is_silent(samples: &[f32]) -> bool {
    samples
        .chunks(FRAME)
        .all(|frame| frame_rms(frame) < SPEECH_FLOOR)
}

/// Join per-window texts: trimmed, empty ones dropped, separated by a space except
/// between two characters of a script written without spaces (Chinese, Japanese).
pub fn join_texts<S: AsRef<str>>(parts: &[S]) -> String {
    let mut out = String::new();
    for part in parts
        .iter()
        .map(|p| p.as_ref().trim())
        .filter(|p| !p.is_empty())
    {
        let joins_without_space = match (out.chars().last(), part.chars().next()) {
            (Some(a), Some(b)) => is_unspaced_script(a) && is_unspaced_script(b),
            _ => true,
        };
        if !joins_without_space {
            out.push(' ');
        }
        out.push_str(part);
    }
    out
}

/// The cut in `lo..hi` (samples): the middle of the longest quiet run touching that
/// range, clamped into it, or the middle of the quietest frame when none is quiet.
fn best_cut(rms: &[f32], quiet: &[bool], lo: usize, hi: usize) -> usize {
    let first = lo.div_ceil(FRAME);
    let last = (hi / FRAME).min(rms.len());
    if first >= last {
        return hi;
    }

    // (run length in frames, run start, run end), all in frames.
    let mut best: Option<(usize, usize, usize)> = None;
    let mut f = first;
    while f < last {
        if !quiet[f] {
            f += 1;
            continue;
        }
        let mut run_start = f;
        while run_start > 0 && quiet[run_start - 1] {
            run_start -= 1;
        }
        let mut run_end = f;
        while run_end < quiet.len() && quiet[run_end] {
            run_end += 1;
        }
        let run_len = run_end - run_start;
        if best.is_none_or(|(len, _, _)| run_len >= len) {
            best = Some((run_len, run_start, run_end));
        }
        f = run_end;
    }

    let cut = match best {
        Some((_, run_start, run_end)) => (run_start + run_end) * FRAME / 2,
        None => {
            let quietest = (first..last)
                .min_by(|&a, &b| rms[a].total_cmp(&rms[b]))
                .unwrap_or(first);
            quietest * FRAME + FRAME / 2
        }
    };
    cut.clamp(lo, hi)
}

/// Quiet below a tenth of the 90th-percentile frame level, never below the floor.
fn quiet_threshold(rms: &[f32]) -> f32 {
    let mut sorted = rms.to_vec();
    sorted.sort_by(f32::total_cmp);
    let loud = sorted[(sorted.len() - 1) * 9 / 10];
    (loud * QUIET_RATIO).max(QUIET_FLOOR)
}

fn frame_rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt()
}

fn is_unspaced_script(c: char) -> bool {
    matches!(c as u32,
        0x3000..=0x303F     // CJK punctuation
        | 0x3040..=0x30FF   // Hiragana, Katakana
        | 0x3400..=0x4DBF   // CJK extension A
        | 0x4E00..=0x9FFF   // CJK unified ideographs
        | 0xF900..=0xFAFF   // CJK compatibility ideographs
        | 0xFF00..=0xFFEF) // full-width forms
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: usize = 16_000;

    fn speech(secs: f32) -> Vec<f32> {
        (0..(secs * SR as f32) as usize)
            .map(|i| 0.3 * (i as f32 * 0.07).sin())
            .collect()
    }

    fn silence(secs: f32) -> Vec<f32> {
        vec![0.0; (secs * SR as f32) as usize]
    }

    fn concat(parts: &[Vec<f32>]) -> Vec<f32> {
        parts.concat()
    }

    fn assert_partition(windows: &[Range<usize>], len: usize, max_secs: f32) {
        assert_eq!(windows.first().map(|w| w.start), Some(0));
        assert_eq!(windows.last().map(|w| w.end), Some(len));
        for pair in windows.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "windows are contiguous");
        }
        for w in windows {
            assert!(!w.is_empty(), "no empty window");
            assert!(
                w.len() <= (max_secs * SR as f32) as usize,
                "window of {} samples over the limit",
                w.len()
            );
        }
    }

    #[test]
    fn short_recording_is_one_window() {
        let audio = speech(3.0);
        assert_eq!(plan_windows(&audio, 6.0), vec![0..audio.len()]);
    }

    #[test]
    fn empty_recording_is_one_empty_window() {
        assert_eq!(plan_windows(&[], 6.0), vec![0..0]);
    }

    #[test]
    fn recording_exactly_at_limit_is_one_window() {
        let audio = speech(6.0);
        assert_eq!(audio.len(), 6 * SR);
        assert_eq!(plan_windows(&audio, 6.0), vec![0..audio.len()]);
    }

    #[test]
    fn recording_just_over_limit_keeps_a_real_tail() {
        let mut audio = speech(6.0);
        audio.push(0.1);
        let windows = plan_windows(&audio, 6.0);
        assert_eq!(windows.len(), 2);
        assert_partition(&windows, audio.len(), 6.0);
        assert!(windows[1].len() >= 6 * SR / 4, "tail is at least max/4");
    }

    #[test]
    fn cuts_land_in_the_pauses() {
        // 4 s speech, 0.6 s pause, 4 s speech, 0.6 s pause, 4 s speech.
        let audio = concat(&[
            speech(4.0),
            silence(0.6),
            speech(4.0),
            silence(0.6),
            speech(4.0),
        ]);
        let windows = plan_windows(&audio, 6.0);
        assert_partition(&windows, audio.len(), 6.0);
        let pauses = [4.0..4.6, 8.6..9.2];
        assert_eq!(windows.len(), 3);
        for (w, pause) in windows.iter().skip(1).zip(pauses) {
            let cut = w.start as f32 / SR as f32;
            assert!(pause.contains(&cut), "cut at {cut:.2}s outside {pause:?}");
        }
    }

    #[test]
    fn longest_pause_wins_over_a_nearer_short_one() {
        // A 1.0 s pause at 3.5 s and a 0.15 s gap at 5.2 s, both in the search range.
        let audio = concat(&[
            speech(3.5),
            silence(1.0),
            speech(0.7),
            silence(0.15),
            speech(5.0),
        ]);
        let windows = plan_windows(&audio, 6.0);
        let cut = windows[1].start as f32 / SR as f32;
        assert!((3.5..4.5).contains(&cut), "cut at {cut:.2}s");
    }

    #[test]
    fn continuous_speech_is_still_cut_within_the_limit() {
        let audio = speech(20.0);
        let windows = plan_windows(&audio, 6.0);
        assert_partition(&windows, audio.len(), 6.0);
        assert!(windows.len() >= 4);
    }

    #[test]
    fn quiet_recording_uses_relative_pauses() {
        // Speech at −40 dBFS with pauses of low noise: cuts still find the pauses.
        let quiet_speech = |secs| speech(secs).iter().map(|s| s * 0.03).collect::<Vec<_>>();
        let noise = |secs: f32| vec![0.0003; (secs * SR as f32) as usize];
        let audio = concat(&[quiet_speech(4.0), noise(0.5), quiet_speech(4.0)]);
        let windows = plan_windows(&audio, 6.0);
        let cut = windows[1].start as f32 / SR as f32;
        assert!((4.0..4.5).contains(&cut), "cut at {cut:.2}s");
    }

    #[test]
    fn all_silence_is_split_and_every_window_is_silent() {
        let audio = silence(20.0);
        let windows = plan_windows(&audio, 6.0);
        assert_partition(&windows, audio.len(), 6.0);
        assert!(windows.iter().all(|w| is_silent(&audio[w.clone()])));
    }

    #[test]
    fn silence_detection() {
        assert!(is_silent(&silence(1.0)));
        assert!(is_silent(&[]));
        assert!(!is_silent(&speech(0.1)));
        let mut mostly_silent = silence(2.0);
        mostly_silent.extend(speech(0.06));
        assert!(!is_silent(&mostly_silent));
    }

    #[test]
    fn long_speech_never_exceeds_a_long_limit() {
        let audio = concat(&[speech(50.0), silence(0.3), speech(50.0)]);
        let windows = plan_windows(&audio, 20.0);
        assert_partition(&windows, audio.len(), 20.0);
    }

    #[test]
    fn join_spaces_words_and_drops_empty_parts() {
        assert_eq!(
            join_texts(&[" Bonjour. ", "", "  ", "Ça va ?"]),
            "Bonjour. Ça va ?"
        );
        assert_eq!(join_texts::<&str>(&[]), "");
        assert_eq!(join_texts(&["", "seul"]), "seul");
    }

    #[test]
    fn join_keeps_unspaced_scripts_together() {
        assert_eq!(join_texts(&["下午三点", "开会。"]), "下午三点开会。");
        assert_eq!(join_texts(&["发送文件。", "OK"]), "发送文件。 OK");
    }
}
