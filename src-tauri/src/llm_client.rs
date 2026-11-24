use crate::settings::PostProcessProvider;
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
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

/// Build headers for API requests based on provider type
fn build_headers(provider: &PostProcessProvider, api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    // Common headers
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://github.com/cjpais/Handy"),
    );
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

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    Ok(completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone()))
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

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

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
    use std::sync::Mutex;

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
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap();
                }
            }
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
        PostProcessProvider {
            id: "mock".to_string(),
            label: "Mock".to_string(),
            base_url: format!("http://127.0.0.1:{port}/v1"),
            allow_base_url_edit: true,
            models_endpoint: None,
            supports_structured_output: true,
        }
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
}
