//! Controlled A/B translation quality + speed benchmark.
//!
//! Loads each GGUF model from the app's models dir and runs the SAME set of
//! source texts through each, using the exact prompt format + sampling the app
//! uses (delimiter/raw completion for TranslateGemma; instruction + chat
//! template for generic models).
//!
//! Measures, per (model, text): generation tok/s, total latency, prompt/gen
//! token counts, whether it hit the token cap (= didn't stop cleanly), and a
//! heuristic "polluted" flag (self-emitted <<<markers>>> or runaway commentary).
//! Reports per-text quality side-by-side + a per-model speed/reliability summary.
//!
//! Run: `cargo run --release --example translation_ab` (app NOT running).

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

struct TestText {
    label: &'static str,
    category: &'static str,
    src: &'static str,
    tgt_code: &'static str,
    tgt_label: &'static str,
    text: &'static str,
}

struct ModelSpec {
    name: &'static str,
    filename: &'static str,
    is_translategemma: bool,
}

#[allow(dead_code)]
struct Outcome {
    output: String,
    gen_tokens: usize,
    prompt_tokens: usize,
    total_ms: u128,
    tok_s: f64,
    hit_cap: bool,
}

fn models_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Library/Application Support/com.dictus.desktop/models")
}

fn build_prompt(spec: &ModelSpec, t: &TestText) -> String {
    if spec.is_translategemma {
        format!(
            "<<<source>>>{}<<<target>>>{}<<<text>>>{}",
            t.src, t.tgt_code, t.text
        )
    } else {
        format!(
            "Translate the following text to {}. Output only the translation, no explanation or commentary.\n\n{}",
            t.tgt_label, t.text
        )
    }
}

fn generate(
    backend: &LlamaBackend,
    model: &LlamaModel,
    prompt: &str,
    apply_template: bool,
) -> Outcome {
    let started = Instant::now();
    let n_threads = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4) as i32)
        / 2;
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()))
        .with_n_threads(n_threads);
    let mut ctx = model.new_context(backend, ctx_params).expect("ctx");

    let formatted = if !apply_template {
        prompt.to_string()
    } else {
        match model.chat_template(None) {
            Ok(tmpl) => {
                let messages =
                    vec![LlamaChatMessage::new("user".to_string(), prompt.to_string()).unwrap()];
                model.apply_chat_template(&tmpl, &messages, true).unwrap()
            }
            Err(_) => prompt.to_string(),
        }
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

    // Bound runaway generation so the benchmark terminates; hitting the cap is
    // itself a signal that the model did not stop cleanly.
    let max_tokens = 256usize;
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
        prompt_tokens,
        total_ms: started.elapsed().as_millis(),
        tok_s,
        hit_cap,
    }
}

/// Heuristic: is the output polluted with self-emitted markers / commentary?
fn is_polluted(out: &str, hit_cap: bool) -> bool {
    hit_cap
        || out.contains("<<<")
        || out.contains("here's the")
        || out.contains("This translation")
        || out.contains("This proverb")
        || out.contains("means that")
}

fn main() {
    let texts = vec![
        TestText { label: "Everyday", category: "easy", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "On se voit demain pour discuter du projet, ça marche pour toi ?" },
        TestText { label: "Technical", category: "tech", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "Le serveur renvoie une erreur 500 quand le cache est invalidé pendant la requête." },
        TestText { label: "Idiom: ours", category: "idiom", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "Il ne faut pas vendre la peau de l'ours avant de l'avoir tué." },
        TestText { label: "Idiom: cordes", category: "idiom", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "Il pleut des cordes, on ferait mieux de rester à la maison." },
        TestText { label: "Idiom: cafard", category: "idiom", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "Depuis qu'il est rentré de vacances, il a le cafard." },
        TestText { label: "Reverse idiom", category: "idiom", src: "en", tgt_code: "fr", tgt_label: "French",
            text: "Don't worry, setting up the project was a piece of cake." },
        TestText { label: "Literary/nuance", category: "literary", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "Le crépuscule jetait sur la ville une lumière mélancolique, comme un adieu murmuré." },
        TestText { label: "Long paragraph", category: "long", src: "fr", tgt_code: "en", tgt_label: "English",
            text: "Aujourd'hui, j'ai passé la matinée à préparer la réunion, puis l'après-midi nous avons discuté du budget prévisionnel pour le trimestre prochain, et même si tout le monde n'était pas d'accord, nous avons fini par trouver un compromis raisonnable avant la fin de la journée." },
        TestText { label: "Dutch source", category: "rare", src: "nl", tgt_code: "en", tgt_label: "English",
            text: "Ondanks de regen hebben we genoten van onze wandeling door de oude binnenstad." },
        TestText { label: "Polish source", category: "rare", src: "pl", tgt_code: "en", tgt_label: "English",
            text: "Mimo że było już późno, postanowiliśmy dokończyć projekt tego samego wieczoru." },
        TestText { label: "Vietnamese source", category: "rare", src: "vi", tgt_code: "en", tgt_label: "English",
            text: "Mặc dù trời mưa to, chúng tôi vẫn quyết định đi bộ đến quán cà phê quen thuộc." },
        TestText { label: "Arabic source", category: "rare", src: "ar", tgt_code: "en", tgt_label: "English",
            text: "على الرغم من صعوبة المشروع، تمكن الفريق من إنهائه قبل الموعد النهائي." },
        TestText { label: "EN->Spanish", category: "target", src: "en", tgt_code: "es", tgt_label: "Spanish",
            text: "Could you send me the report before the meeting tomorrow morning?" },
        TestText { label: "EN->German", category: "target", src: "en", tgt_code: "de", tgt_label: "German",
            text: "We really appreciate your patience while we fix the remaining issues." },
    ];

    let models = vec![
        ModelSpec {
            name: "TranslateGemma-4B",
            filename: "translategemma-4b-it-Q4_K_M.gguf",
            is_translategemma: true,
        },
        ModelSpec {
            name: "Gemma-3-4B",
            filename: "gemma-3-4b-it-Q4_K_M.gguf",
            is_translategemma: false,
        },
        ModelSpec {
            name: "Llama-3.2-3B",
            filename: "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
            is_translategemma: false,
        },
        ModelSpec {
            name: "Qwen2.5-1.5B",
            filename: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
            is_translategemma: false,
        },
    ];

    let backend = LlamaBackend::init().expect("backend");
    let dir = models_dir();

    // results[text_idx][model_idx]
    let mut results: Vec<Vec<Option<Outcome>>> = (0..texts.len()).map(|_| Vec::new()).collect();
    // per-model aggregate: (load_ms, sum_tok_s, n, polluted_count, hit_cap_count)
    let mut load_times: Vec<u128> = Vec::new();

    for spec in &models {
        let path = dir.join(spec.filename);
        if !path.exists() {
            eprintln!("SKIP {}: not found", spec.name);
            for r in results.iter_mut() {
                r.push(None);
            }
            load_times.push(0);
            continue;
        }
        eprintln!("Loading {} ...", spec.name);
        let load_started = Instant::now();
        let params = LlamaModelParams::default().with_n_gpu_layers(u32::MAX);
        let model = LlamaModel::load_from_file(&backend, &path, &params).expect("load");
        load_times.push(load_started.elapsed().as_millis());
        for (i, t) in texts.iter().enumerate() {
            let prompt = build_prompt(spec, t);
            let outcome = generate(&backend, &model, &prompt, !spec.is_translategemma);
            eprintln!(
                "  {} · {} -> {} gen tok, {:.0} tok/s, {} ms{}",
                spec.name,
                t.label,
                outcome.gen_tokens,
                outcome.tok_s,
                outcome.total_ms,
                if outcome.hit_cap { " [HIT CAP]" } else { "" }
            );
            results[i].push(Some(outcome));
        }
        drop(model);
    }

    // ── Per-text quality side-by-side ──
    println!("\n\n================ TRANSLATION QUALITY (per text) ================\n");
    for (i, t) in texts.iter().enumerate() {
        println!("──────────────────────────────────────────────────────");
        println!("[{}] {}  ({} → {})", t.category, t.label, t.src, t.tgt_code);
        println!("SRC: {}", t.text);
        println!();
        for (j, spec) in models.iter().enumerate() {
            match &results[i][j] {
                Some(o) => {
                    let flag = if is_polluted(&o.output, o.hit_cap) {
                        " ⚠ POLLUTED"
                    } else {
                        ""
                    };
                    println!("  [{}]{}", spec.name, flag);
                    // Truncate very long (runaway) outputs in the quality view.
                    let shown: String = o.output.chars().take(400).collect();
                    let ell = if o.output.chars().count() > 400 {
                        " …[truncated]"
                    } else {
                        ""
                    };
                    println!("    {}{}", shown.replace('\n', " "), ell);
                }
                None => println!("  [{}] (not available)", spec.name),
            }
            println!();
        }
    }

    // ── Per-model speed + reliability summary ──
    println!("\n================ SPEED & RELIABILITY SUMMARY ================\n");
    println!(
        "{:<22} {:>10} {:>12} {:>12} {:>10} {:>10}",
        "Model", "load(ms)", "avg tok/s", "avg lat(ms)", "polluted", "hit-cap"
    );
    for (j, spec) in models.iter().enumerate() {
        let outs: Vec<&Outcome> = results.iter().filter_map(|r| r[j].as_ref()).collect();
        if outs.is_empty() {
            println!("{:<22} {:>10}", spec.name, "n/a");
            continue;
        }
        let n = outs.len() as f64;
        let avg_tok_s = outs.iter().map(|o| o.tok_s).sum::<f64>() / n;
        let avg_lat = outs.iter().map(|o| o.total_ms as f64).sum::<f64>() / n;
        let polluted = outs
            .iter()
            .filter(|o| is_polluted(&o.output, o.hit_cap))
            .count();
        let hitcap = outs.iter().filter(|o| o.hit_cap).count();
        println!(
            "{:<22} {:>10} {:>12.1} {:>12.0} {:>8}/{} {:>8}/{}",
            spec.name,
            load_times[j],
            avg_tok_s,
            avg_lat,
            polluted,
            outs.len(),
            hitcap,
            outs.len()
        );
    }
    println!("\n(avg lat = full per-call latency incl. context build + prompt eval; load = one-time cold model load)");
}
