use crate::settings::PostProcessProvider;
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    strict: bool,
    schema: Value,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
    json_schema: JsonSchema,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

/// Parameters for a chat completion request with optional structured output support.
/// Derives `Default` so callers can use `..Default::default()` for unset fields.
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionParams {
    pub provider: PostProcessProvider,
    pub api_key: String,
    pub model: String,
    pub user_content: String,
    pub system_prompt: Option<String>,
    pub json_schema: Option<Value>,
    pub reasoning_effort: Option<String>,
    pub reasoning: Option<ReasoningConfig>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

const DICTUS_URL: &str = "https://getdictus.com";

/// Build headers for API requests based on provider type
fn build_headers(provider: &PostProcessProvider, api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    // Common headers
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    // OpenRouter attributes usage to the app named by `HTTP-Referer`, not the
    // standard `Referer`, and shows `X-Title` as its display name.
    headers.insert(REFERER, HeaderValue::from_static(DICTUS_URL));
    headers.insert("HTTP-Referer", HeaderValue::from_static(DICTUS_URL));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)"),
    );
    headers.insert("X-Title", HeaderValue::from_static("Dictus"));

    // Provider-specific auth headers
    if !api_key.is_empty() {
        if provider.id == "anthropic" {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        } else {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| format!("Invalid authorization header value: {}", e))?,
            );
        }
    }

    Ok(headers)
}

/// Create an HTTP client with provider-specific headers
fn create_client(provider: &PostProcessProvider, api_key: &str) -> Result<reqwest::Client, String> {
    let headers = build_headers(provider, api_key)?;
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Send a chat completion request to an OpenAI-compatible API
/// Returns Ok(Some(content)) on success, Ok(None) if response has no content,
/// or Err on actual errors (HTTP, parsing, etc.)
pub async fn send_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    prompt: String,
    reasoning_effort: Option<String>,
    reasoning: Option<ReasoningConfig>,
) -> Result<Option<String>, String> {
    send_chat_completion_with_schema(ChatCompletionParams {
        provider: provider.clone(),
        api_key,
        model: model.to_string(),
        user_content: prompt,
        reasoning_effort,
        reasoning,
        ..Default::default()
    })
    .await
}

/// Send a chat completion request with structured output support
/// When json_schema is provided, uses structured outputs mode
/// system_prompt is used as the system message when provided
/// reasoning_effort sets the OpenAI-style top-level field (e.g., "none", "low", "medium", "high")
/// reasoning sets the OpenRouter-style nested object (effort + exclude)
pub async fn send_chat_completion_with_schema(
    params: ChatCompletionParams,
) -> Result<Option<String>, String> {
    let ChatCompletionParams {
        provider,
        api_key,
        model,
        user_content,
        system_prompt,
        json_schema,
        reasoning_effort,
        reasoning,
    } = params;

    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);

    debug!("Sending chat completion request to: {}", url);

    let client = create_client(&provider, &api_key)?;

    // Build messages vector
    let mut messages = Vec::new();

    // Add system prompt if provided
    if let Some(system) = system_prompt {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system,
        });
    }

    // Add user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content,
    });

    // Build response_format if schema is provided
    let response_format = json_schema.map(|schema| ResponseFormat {
        format_type: "json_schema".to_string(),
        json_schema: JsonSchema {
            name: "transcription_output".to_string(),
            strict: true,
            schema,
        },
    });

    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        response_format,
        reasoning_effort,
        reasoning,
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(http_error("API request", &provider.id, response).await);
    }

    let completion: ChatCompletionResponse =
        parse_body("API response", &provider.id, response).await?;

    Ok(completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone()))
}

/// The error for a non-2xx response, without the response body (#96).
/// Providers echo the request in their error bodies, transcript included, so the
/// body is logged only through `debug_user_text!`. The error keeps the status, the
/// body size and the provider's error type and code when they are identifiers.
async fn http_error(request: &str, provider_id: &str, response: reqwest::Response) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    crate::debug_user_text!(
        "{} error body from provider '{}': {}",
        request,
        provider_id,
        body
    );
    let mut error = format!(
        "{} failed with status {} (provider '{}', {} bytes",
        request,
        status,
        provider_id,
        body.len()
    );
    let details = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|json| json.get("error").cloned());
    for field in ["type", "code"] {
        if let Some(token) = details
            .as_ref()
            .and_then(|details| details.get(field))
            .and_then(error_token)
        {
            error.push_str(&format!(", {} {}", field, token));
        }
    }
    error.push(')');
    error
}

/// Parse a 2xx response body, keeping the body out of the error (#105).
/// serde_json's message can quote a value from the body (`invalid type: string
/// "…"`) and a chat completion carries the dictated text, so the error keeps the
/// status, the body size and the parse error's category and position. The full
/// message is logged only through `debug_user_text!`.
async fn parse_body<T: DeserializeOwned>(
    response_name: &str,
    provider_id: &str,
    response: reqwest::Response,
) -> Result<T, String> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read {}: {}", response_name, e))?;
    serde_json::from_str(&body).map_err(|e| {
        crate::debug_user_text!(
            "{} from provider '{}' does not parse: {}. Body: {}",
            response_name,
            provider_id,
            e,
            body
        );
        let category = match e.classify() {
            Category::Io => "I/O",
            Category::Syntax => "syntax",
            Category::Data => "data",
            Category::Eof => "end of input",
        };
        format!(
            "Failed to parse {} with status {} (provider '{}', {} bytes, {} error at line {} column {})",
            response_name,
            status,
            provider_id,
            body.len(),
            category,
            e.line(),
            e.column()
        )
    })
}

/// A provider error type or code, kept only when it is an identifier, never prose.
fn error_token(value: &Value) -> Option<String> {
    let token = match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    let is_identifier = !token.is_empty()
        && token.len() <= 64
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    is_identifier.then_some(token)
}

/// Fetch available models from an OpenAI-compatible API
/// Returns a list of model IDs
pub async fn fetch_models(
    provider: &PostProcessProvider,
    api_key: String,
) -> Result<Vec<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/models", base_url);

    debug!("Fetching models from: {}", url);

    let client = create_client(provider, &api_key)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    if !response.status().is_success() {
        return Err(http_error("Model list request", &provider.id, response).await);
    }

    let parsed: Value = parse_body("Model list response", &provider.id, response).await?;

    let mut models = Vec::new();

    // Handle OpenAI format: { data: [ { id: "..." }, ... ] }
    if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
        for entry in data {
            if let Some(id) = entry.get("id").and_then(|i| i.as_str()) {
                models.push(id.to_string());
            } else if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                models.push(name.to_string());
            }
        }
    }
    // Handle array format: [ "model1", "model2", ... ]
    else if let Some(array) = parsed.as_array() {
        for entry in array {
            if let Some(model) = entry.as_str() {
                models.push(model.to_string());
            }
        }
    }

    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{mpsc, Mutex};

    const DICTATED: &str = "meet Camille at the pharmacy at nine";

    struct Capture(Mutex<Vec<(log::Level, String)>>);

    impl log::Log for Capture {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, record: &log::Record) {
            self.0
                .lock()
                .unwrap()
                .push((record.level(), record.args().to_string()));
        }
        fn flush(&self) {}
    }

    static CAPTURE: Capture = Capture(Mutex::new(Vec::new()));

    /// The tests share the logger and the "Log transcripts" switch.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn lines_with_dictated_text() -> Vec<(log::Level, String)> {
        CAPTURE
            .0
            .lock()
            .unwrap()
            .drain(..)
            .filter(|(_, line)| line.contains(DICTATED))
            .collect()
    }

    /// A provider that answers one request with `status` and a body built from
    /// the request body, the way validation and content-filter errors echo it.
    fn mock_provider(status: &'static str, reply: fn(&str) -> String) -> PostProcessProvider {
        mock_server(status, reply).0
    }

    /// Like `mock_provider`, and also hands back the request's header lines.
    fn mock_server(
        status: &'static str,
        reply: fn(&str) -> String,
    ) -> (PostProcessProvider, mpsc::Receiver<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (headers_tx, headers_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut length = 0;
            let mut headers = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap();
                }
                headers.push(line.trim_end().to_string());
            }
            let _ = headers_tx.send(headers);
            let mut request = vec![0; length];
            reader.read_exact(&mut request).unwrap();
            let body = reply(&String::from_utf8(request).unwrap());
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });
        let provider = PostProcessProvider {
            id: "mock".to_string(),
            label: "Mock".to_string(),
            base_url: format!("http://127.0.0.1:{port}/v1"),
            allow_base_url_edit: true,
            models_endpoint: None,
            supports_structured_output: true,
        };
        (provider, headers_rx)
    }

    fn openai_echo(request: &str) -> String {
        serde_json::json!({
            "error": {
                "message": format!("Invalid request: {request}"),
                "type": "invalid_request_error",
                "code": "context_length_exceeded",
            }
        })
        .to_string()
    }

    fn prose_code_echo(request: &str) -> String {
        serde_json::json!({ "error": { "type": 400, "code": format!("bad input {request}") } })
            .to_string()
    }

    fn plain_echo(request: &str) -> String {
        format!("rejected: {request} {DICTATED}")
    }

    /// A 2xx chat completion whose `message` is a bare string instead of an
    /// object, so serde's type error quotes the dictated text.
    fn wrong_type_echo(_: &str) -> String {
        serde_json::json!({ "choices": [{ "message": DICTATED }] }).to_string()
    }

    fn chat(provider: PostProcessProvider) -> String {
        tauri::async_runtime::block_on(send_chat_completion_with_schema(ChatCompletionParams {
            provider,
            api_key: "key".to_string(),
            model: "m".to_string(),
            user_content: DICTATED.to_string(),
            system_prompt: Some("Clean up the transcript.".to_string()),
            ..Default::default()
        }))
        .unwrap_err()
    }

    #[test]
    fn http_error_bodies_stay_out_of_errors_and_logs() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let _ = log::set_logger(&CAPTURE);
        log::set_max_level(log::LevelFilter::Trace);

        crate::logging::set_log_transcripts(false);
        let err = chat(mock_provider("400 Bad Request", openai_echo));
        assert!(
            !err.contains(DICTATED),
            "chat error carries the body: {err}"
        );
        assert!(err.contains("400"), "{err}");
        assert!(err.contains("'mock'"), "{err}");
        assert!(err.contains("bytes"), "{err}");
        assert!(err.contains("invalid_request_error"), "{err}");
        assert!(err.contains("context_length_exceeded"), "{err}");
        assert_eq!(lines_with_dictated_text(), vec![]);

        let err = chat(mock_provider("400 Bad Request", prose_code_echo));
        assert!(!err.contains(DICTATED), "prose error code kept: {err}");
        assert!(!err.contains("bad input"), "prose error code kept: {err}");

        let err = tauri::async_runtime::block_on(fetch_models(
            &mock_provider("401 Unauthorized", plain_echo),
            "key".to_string(),
        ))
        .unwrap_err();
        assert!(
            !err.contains(DICTATED),
            "models error carries the body: {err}"
        );
        assert!(err.contains("401"), "{err}");
        assert_eq!(lines_with_dictated_text(), vec![]);

        crate::logging::set_log_transcripts(true);
        chat(mock_provider("400 Bad Request", openai_echo));
        crate::logging::set_log_transcripts(false);
        let lines = lines_with_dictated_text();
        assert!(
            !lines.is_empty(),
            "Log transcripts on: body logged at Debug"
        );
        assert!(
            lines.iter().all(|(level, _)| *level == log::Level::Debug),
            "{lines:?}"
        );
    }

    #[test]
    fn unparseable_2xx_bodies_stay_out_of_errors_and_logs() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let _ = log::set_logger(&CAPTURE);
        log::set_max_level(log::LevelFilter::Trace);

        crate::logging::set_log_transcripts(false);
        let err = chat(mock_provider("200 OK", wrong_type_echo));
        assert!(
            !err.contains(DICTATED),
            "parse error carries the body: {err}"
        );
        assert!(err.contains("200"), "{err}");
        assert!(err.contains("'mock'"), "{err}");
        assert!(err.contains("bytes"), "{err}");
        assert!(err.contains("data error at line 1 column"), "{err}");
        assert_eq!(lines_with_dictated_text(), vec![]);

        crate::logging::set_log_transcripts(true);
        chat(mock_provider("200 OK", wrong_type_echo));
        crate::logging::set_log_transcripts(false);
        let lines = lines_with_dictated_text();
        assert!(
            !lines.is_empty(),
            "Log transcripts on: parse error logged at Debug"
        );
        assert!(
            lines.iter().all(|(level, _)| *level == log::Level::Debug),
            "{lines:?}"
        );
    }

    fn ok_reply(_: &str) -> String {
        serde_json::json!({ "choices": [{ "message": { "content": "ok" } }] }).to_string()
    }

    fn header<'a>(headers: &'a [String], name: &str) -> Option<&'a str> {
        headers.iter().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name).then(|| value.trim())
        })
    }

    #[test]
    fn attribution_headers_name_dictus() {
        let headers = build_headers(&PostProcessProvider::default(), "").unwrap();
        for name in ["referer", "http-referer"] {
            assert_eq!(headers[name], "https://getdictus.com", "{name}");
        }
        assert_eq!(headers["x-title"], "Dictus");
    }

    #[test]
    fn provider_receives_dictus_attribution() {
        let (provider, headers) = mock_server("200 OK", ok_reply);
        let reply = tauri::async_runtime::block_on(send_chat_completion(
            &provider,
            "key".to_string(),
            "m",
            "hello".to_string(),
            None,
            None,
        ));
        assert_eq!(reply, Ok(Some("ok".to_string())));

        let headers = headers.recv().unwrap();
        println!("headers received by the mock provider:");
        for line in headers
            .iter()
            .filter(|line| !line.starts_with("authorization"))
        {
            println!("  {line}");
        }
        assert_eq!(header(&headers, "referer"), Some("https://getdictus.com"));
        assert_eq!(
            header(&headers, "http-referer"),
            Some("https://getdictus.com")
        );
        assert_eq!(header(&headers, "x-title"), Some("Dictus"));
        assert!(
            !headers
                .iter()
                .any(|line| line.to_ascii_lowercase().contains("handy")),
            "{headers:?}"
        );
    }
}
