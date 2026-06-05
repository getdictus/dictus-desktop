//! Smart Mode REWRITE prompt iteration harness.
//!
//! Faithfully reproduces the app's EMBEDDED post-processing path
//! (`actions.rs::post_process_with_prompt`, embedded branch):
//!
//!   base   = prompt_template with `${output}` stripped + trimmed
//!   dir    = language_directive(text)  (whatlang-detected, same wording as app)
//!   system = format!("{dir}\n\n{base}")
//!   full   = format!("{system}\n\n{text}")     // single user turn, no role split
//!   -> chat template applied, sampler = penalties(64,1.1)+greedy, max 512, ctx 2048
//!
//! Edit the PROMPTS + INPUTS tables below, then:
//!   `cargo run --release --example rewrite_modes`   (app NOT running)
//!
//! Loads the active model (gemma-3-4b) only, to keep the loop fast.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

const MODEL_FILE: &str = "gemma-3-4b-it-Q4_K_M.gguf";

struct Mode {
    id: &'static str,
    label: &'static str,
    /// Verbatim default prompt template (keep the `${output}` placeholder).
    prompt: &'static str,
}

struct Case {
    /// which Mode.id this input is meant to exercise
    mode_id: &'static str,
    label: &'static str,
    text: &'static str,
}

// ─────────────────────────────────────────────────────────────────────────
// PROMPTS — current defaults from settings.rs::smart_mode_templates().
// Edit these to iterate.
// ─────────────────────────────────────────────────────────────────────────
fn modes() -> Vec<Mode> {
    vec![
        Mode {
            id: "clean_up",
            label: "Clean Up",
            prompt: "Clean this transcript:\n1. Fix spelling, capitalization, and punctuation errors\n2. ALWAYS write numbers and quantities as digits, never spelled out. Convert every single one: vingt-cinq mille euros → 25 000 €, dix pour cent → 10 %, quatorze heures → 14 h, trois → 3, twenty-five → 25, ten percent → 10%\n3. Replace spoken punctuation with symbols (point/period → ., virgule/comma → ,, point d'interrogation/question mark → ?)\n4. Remove filler words (euh, um, uh, like as filler)\n5. Keep the language in the original version (if it was french, keep it in french for example)\n6. Never output two punctuation marks in a row (for example \".,\" or \",.\")\n\nPreserve exact meaning and word order. Do not paraphrase or reorder content.\n\nReturn only the cleaned transcript.\n\nTranscript:\n${output}",
        },
        Mode {
            id: "make_formal",
            label: "Make Formal",
            prompt: "Rewrite this text in a formal, professional tone while preserving its exact meaning and language. Return only the rewritten text.\n\nText:\n${output}",
        },
        Mode {
            id: "make_casual",
            label: "Make Casual",
            prompt: "Rewrite this text in a relaxed, casual, conversational tone while preserving its exact meaning and language. Return only the rewritten text.\n\nText:\n${output}",
        },
        Mode {
            id: "email",
            label: "Write as Email",
            prompt: "Reformat this text as an email WITHOUT changing its tone, register, or wording more than necessary. Keep exactly the same level of familiarity or formality as the input: if it is casual and friendly, the email stays casual and friendly; if it is formal (vouvoiement), it stays formal. Apply ONLY email structure:\n- Start with exactly ONE greeting line, then a blank line. Use the same greeting word the user actually spoke and do not change it: if the text starts with \"salut\", the greeting must start with \"Salut\"; if it starts with \"bonjour\", it must start with \"Bonjour\". Add the recipient's name and fix capitalization. If the input has no greeting at all, use \"Bonjour,\". Never write a second greeting.\n- Then the message body, fixing capitalization and punctuation and splitting the run-on dictation into proper sentences (add periods and question marks where needed). Keep the user's own words and phrasing; do NOT make requests more polite and do NOT add words the user did not say (for example, never add \"s'il te plaît\" or \"please\").\n- Closing: ONLY if the input ends with a closing phrase (such as \"cordialement\", \"bien cordialement\", \"à plus\", \"bien à vous\", \"merci\") followed by a name, you MUST move it OUT of the body into a separate signature block: end the body sentence, then a blank line, then the closing word on its own line, then the name on the next line. Example — input body \"...votre email ? Bien cordialement, Pierre.\" must become:\n...votre email ?\n\nBien cordialement,\nPierre\nIf the input contains no such closing, do NOT add one.\nDo NOT add a subject line. Do NOT invent pleasantries such as \"I hope you are well\". Do not add, remove, or reword the actual content. Preserve the meaning and language. Return only the email.\n\nText:\n${output}",
        },
        Mode {
            id: "bullet_points",
            label: "Bullet Points",
            prompt: "Restructure this text as a concise bulleted list, one idea per bullet, preserving meaning and language. Use a hyphen \"- \" as the bullet marker for every item. Return only the bullet list.\n\nText:\n${output}",
        },
        Mode {
            id: "summarize",
            label: "Summarize",
            prompt: "Summarize this text concisely, preserving the key points and language. Return only the summary.\n\nText:\n${output}",
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────
// INPUTS — realistic raw French dictation transcripts (what STT produces).
// ─────────────────────────────────────────────────────────────────────────
fn cases() -> Vec<Case> {
    vec![
        Case {
            mode_id: "clean_up",
            label: "Rambling planning",
            text: "alors euh donc en gros le truc c'est que faut qu'on revoie le planning parce que voilà euh ça va pas le faire pour vendredi tu vois donc euh je pense qu'on devrait décaler à lundi prochain",
        },
        Case {
            mode_id: "clean_up",
            label: "Numbers & spoken punctuation",
            text: "du coup le budget il est de vingt cinq mille euros environ et on a déjà dépensé genre dix pour cent point on en reparle demain point d'interrogation",
        },
        Case {
            // Real case from logs: dictated !, ?, and line-break cues.
            mode_id: "clean_up",
            label: "Exclamation & line breaks (real)",
            text: "Hello, je voulais juste te dire que je ne suis plus sûr du clean up qu'on a fait, point retour à la ligne, qu'on devrait peut-être faire autrement, point d'exclamation, retour à la ligne, qu'en penses-tu, point d'interrogation.",
        },
        Case {
            mode_id: "make_formal",
            label: "Casual ask to client",
            text: "salut faut absolument qu'on cale un call cette semaine pour parler du contrat c'est un peu chaud niveau délais",
        },
        Case {
            mode_id: "make_casual",
            label: "Stiff message",
            text: "Je me permets de vous solliciter afin de convenir d'un entretien téléphonique à votre meilleure convenance concernant l'avancement du projet.",
        },
        Case {
            mode_id: "email",
            label: "Casual, no signature",
            text: "salut tom est ce que tu peux m'envoyer le rapport trimestriel avant la réunion de demain matin parce que j'aimerais le relire avant merci beaucoup",
        },
        Case {
            mode_id: "email",
            label: "With dictated signature",
            text: "bonjour madame durand suite à notre échange je vous confirme ma disponibilité pour le rendez vous de jeudi à quatorze heures cordialement pierre vivière",
        },
        Case {
            // Real failing case from logs: closing glued to the body sentence.
            mode_id: "email",
            label: "Inline closing (real)",
            text: "Bonjour madame, pourriez-vous m'envoyer votre email s'il vous plaît ? Bien cordialement, Pierre.",
        },
        Case {
            mode_id: "email",
            label: "Casual closing (real)",
            text: "Salut, est-ce que tu as fait le document que je t'ai demandé hier ? A plus, Pierre.",
        },
        Case {
            mode_id: "bullet_points",
            label: "Meeting actions",
            text: "donc pour la réunion il faut que je prépare le slide deck il faut aussi que j'appelle le fournisseur pour les délais et puis ne pas oublier de relancer la compta pour la facture en retard et enfin réserver la salle pour jeudi",
        },
        Case {
            mode_id: "summarize",
            label: "Long status update",
            text: "aujourd'hui j'ai passé la matinée à préparer la réunion puis l'après midi on a discuté du budget prévisionnel pour le trimestre prochain et même si tout le monde n'était pas d'accord on a fini par trouver un compromis raisonnable avant la fin de la journée mais il reste encore à valider avec la direction la semaine prochaine",
        },
    ]
}

// ── app-faithful prompt assembly (embedded path) ──────────────────────────

fn detect_language_name(text: &str) -> Option<String> {
    if text.trim().chars().count() < 8 {
        return None;
    }
    whatlang::detect(text).map(|info| info.lang().eng_name().to_string())
}

fn language_directive(text: &str) -> Option<String> {
    detect_language_name(text).map(|lang| {
        format!(
            "Write your entire response in {} (the language of the text being processed), unless the task explicitly asks you to translate or to use another language. Do not mention, repeat, or include this instruction in your output.",
            lang
        )
    })
}

fn build_full_prompt(template: &str, text: &str) -> String {
    let base = template.replace("${output}", "").trim().to_string();
    let system = match language_directive(text) {
        Some(dir) => format!("{}\n\n{}", dir, base),
        None => base,
    };
    format!("{}\n\n{}", system, text)
}

fn models_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Library/Application Support/com.dictus.desktop/models")
}

struct Outcome {
    output: String,
    gen_tokens: usize,
    total_ms: u128,
    tok_s: f64,
    hit_cap: bool,
}

fn generate(backend: &LlamaBackend, model: &LlamaModel, user_msg: &str) -> Outcome {
    let started = Instant::now();
    let n_threads = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) as i32)
        / 2;
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()))
        .with_n_threads(n_threads);
    let mut ctx = model.new_context(backend, ctx_params).expect("ctx");

    let formatted = match model.chat_template(None) {
        Ok(tmpl) => {
            let messages =
                vec![LlamaChatMessage::new("user".to_string(), user_msg.to_string()).unwrap()];
            model.apply_chat_template(&tmpl, &messages, true).unwrap()
        }
        Err(_) => user_msg.to_string(),
    };

    let tokens_list = model.str_to_token(&formatted, AddBos::Always).expect("tok");
    let prompt_tokens = tokens_list.len();
    let mut batch = LlamaBatch::new(2048, 1);
    for (i, token) in tokens_list.iter().enumerate() {
        batch
            .add(*token, i as i32, &[0], i == prompt_tokens - 1)
            .unwrap();
    }
    ctx.decode(&mut batch).unwrap();

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
        LlamaSampler::greedy(),
    ]);

    let max_tokens = 512usize;
    let mut output = String::new();
    let mut pos = prompt_tokens as i32;
    let mut n_generated = 0usize;
    let mut hit_cap = true;
    let gen_started = Instant::now();

    for _ in 0..max_tokens {
        let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(new_token);
        if model.is_eog_token(new_token) {
            hit_cap = false;
            break;
        }
        let token_bytes = model
            .token_to_piece_bytes(new_token, 64, false, None)
            .unwrap();
        output.push_str(&String::from_utf8_lossy(&token_bytes));
        n_generated += 1;
        batch.clear();
        batch.add(new_token, pos, &[0], true).unwrap();
        ctx.decode(&mut batch).unwrap();
        pos += 1;
    }

    let gen_secs = gen_started.elapsed().as_secs_f64();
    let tok_s = if gen_secs > 0.0 {
        n_generated as f64 / gen_secs
    } else {
        0.0
    };
    Outcome {
        output: output.trim().to_string(),
        gen_tokens: n_generated,
        total_ms: started.elapsed().as_millis(),
        tok_s,
        hit_cap,
    }
}

fn main() {
    let modes = modes();
    let cases = cases();

    let backend = LlamaBackend::init().expect("backend");
    let path = models_dir().join(MODEL_FILE);
    if !path.exists() {
        eprintln!("Model not found: {}", path.display());
        std::process::exit(1);
    }
    eprintln!("Loading {} ...", MODEL_FILE);
    let load_started = Instant::now();
    let params = LlamaModelParams::default().with_n_gpu_layers(u32::MAX);
    let model = LlamaModel::load_from_file(&backend, &path, &params).expect("load");
    eprintln!("Loaded in {} ms\n", load_started.elapsed().as_millis());

    println!("================ SMART MODE REWRITE — gemma-3-4b ================\n");

    for case in &cases {
        let mode = modes.iter().find(|m| m.id == case.mode_id).unwrap();
        let full = build_full_prompt(mode.prompt, case.text);
        let out = generate(&backend, &model, &full);
        let flag = if out.hit_cap { "  ⚠ HIT CAP" } else { "" };

        println!("──────────────────────────────────────────────────────────────");
        println!("MODE: {}  ·  CASE: {}{}", mode.label, case.label, flag);
        println!("IN : {}", case.text);
        println!("OUT: {}", out.output.replace('\n', "\n     "));
        println!(
            "     [{} tok, {:.0} tok/s, {} ms]",
            out.gen_tokens, out.tok_s, out.total_ms
        );
        println!();
    }
}
