//! Dictated text never reaches the logs by default (#83).
//!
//! A transcript, an LLM output or anything typed or pasted for the user is logged
//! only through [`debug_user_text!`]: at Debug level, and only while the user has
//! turned on Debug → "Log transcripts". Every other log line carries lengths,
//! durations, model ids and timings. The test at the bottom fails the build when a
//! plain log macro formats one of the text variables.

use std::sync::atomic::{AtomicBool, Ordering};

static LOG_TRANSCRIPTS: AtomicBool = AtomicBool::new(false);

pub fn set_log_transcripts(enabled: bool) {
    LOG_TRANSCRIPTS.store(enabled, Ordering::Relaxed);
}

pub fn transcripts_logged() -> bool {
    LOG_TRANSCRIPTS.load(Ordering::Relaxed)
}

/// `log::debug!` for a line that carries user text, emitted only while the user
/// has turned on transcript logging.
#[macro_export]
macro_rules! debug_user_text {
    ($($arg:tt)+) => {
        if $crate::logging::transcripts_logged() {
            ::log::debug!($($arg)+);
        }
    };
}

/// Dependency targets that log the text they handle, with the most verbose level
/// they may log at. enigo logs every typed string at Debug (`fast_text(text: …)`)
/// and every key at Trace; transcribe-rs's Canary engine logs its transcript at Info.
pub const TEXT_ECHOING_TARGETS: &[(&str, log::LevelFilter)] = &[
    ("enigo", log::LevelFilter::Info),
    ("transcribe_rs::onnx::canary", log::LevelFilter::Warn),
];

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// Variables that hold user text somewhere in the backend.
    const TEXT_VARS: &[&str] = &[
        "transcription",
        "transcript",
        "final_result",
        "filtered_result",
        "corrected_result",
        "final_text",
        "transcription_text",
        "post_processed_text",
        "processed_text",
        "text",
        "trimmed",
        "output",
        "content",
        "stdout",
        "stderr",
    ];

    /// A text variable is fine when only its size or emptiness is logged.
    const SIZE_SUFFIXES: &[&str] = &[".len()", ".chars().count()", ".is_empty()"];

    const LOG_MACROS: &[&str] = &["trace!", "debug!", "info!", "warn!", "error!"];

    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// The text between the macro's opening parenthesis and its matching close.
    fn macro_args(src: &str, open: usize) -> &str {
        let mut depth = 0;
        let mut in_str = false;
        let mut escaped = false;
        for (i, c) in src[open..].char_indices() {
            if in_str {
                match (escaped, c) {
                    (true, _) => escaped = false,
                    (false, '\\') => escaped = true,
                    (false, '"') => in_str = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return &src[open + 1..open + i];
                    }
                }
                _ => {}
            }
        }
        &src[open + 1..]
    }

    /// The format string's inline captures (`{text}`) and the arguments after it,
    /// with string literals removed.
    fn formatted_code(args: &str) -> String {
        let mut code = String::new();
        let mut in_str = false;
        let mut escaped = false;
        let mut literal = String::new();
        for c in args.chars() {
            if in_str {
                match (escaped, c) {
                    (true, _) => escaped = false,
                    (false, '\\') => escaped = true,
                    (false, '"') => {
                        in_str = false;
                        for capture in literal.split('{').skip(1) {
                            let name = capture.split(['}', ':']).next().unwrap_or("");
                            code.push(' ');
                            code.push_str(name);
                            code.push(' ');
                        }
                        literal.clear();
                    }
                    _ => literal.push(c),
                }
                continue;
            }
            if c == '"' {
                in_str = true;
                code.push(' ');
            } else {
                code.push(c);
            }
        }
        code
    }

    fn logs_text(code: &str) -> Option<&'static str> {
        let is_ident = |c: char| c.is_alphanumeric() || c == '_';
        for var in TEXT_VARS {
            for (at, _) in code.match_indices(var) {
                let before = code[..at].chars().next_back();
                let after = &code[at + var.len()..];
                let whole_word = !before.is_some_and(|c| is_ident(c) || c == '.')
                    && !after.chars().next().is_some_and(is_ident);
                if whole_word && !SIZE_SUFFIXES.iter().any(|s| after.starts_with(s)) {
                    return Some(var);
                }
            }
        }
        None
    }

    #[test]
    fn no_log_macro_formats_user_text_outside_debug_user_text() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_files(&root, &mut files);
        let mut leaks = Vec::new();
        for file in &files {
            let src = std::fs::read_to_string(file).unwrap();
            for mac in LOG_MACROS {
                for (at, _) in src.match_indices(mac) {
                    let prefix = &src[..at];
                    if prefix.ends_with("debug_user_text!") || prefix.ends_with('_') {
                        continue;
                    }
                    let rest = &src[at + mac.len()..];
                    if !rest.trim_start().starts_with('(') {
                        continue;
                    }
                    let open = at + mac.len() + rest.find('(').unwrap();
                    let args = macro_args(&src, open);
                    if let Some(var) = logs_text(&formatted_code(args)) {
                        let line = prefix.matches('\n').count() + 1;
                        let rel = file.strip_prefix(&root).unwrap().display();
                        leaks.push(format!("src/{rel}:{line} {mac} formats `{var}`"));
                    }
                }
            }
        }
        assert!(
            leaks.is_empty(),
            "log macros that format user text; log its length, or use debug_user_text!:\n{}",
            leaks.join("\n")
        );
    }

    #[test]
    fn detector_sees_text_and_ignores_sizes() {
        let code = |args: &str| formatted_code(args);
        assert_eq!(
            logs_text(&code(r#""Transcription result: {}", final_result"#)),
            Some("final_result")
        );
        assert_eq!(logs_text(&code(r#""typed {text}""#)), Some("text"));
        assert_eq!(logs_text(&code(r#""{} chars", final_result.len()"#)), None);
        assert_eq!(
            logs_text(&code(r#""Using xdotool for direct text input""#)),
            None
        );
        assert_eq!(logs_text(&code(r#""{}", entry.text_len"#)), None);
    }
}
