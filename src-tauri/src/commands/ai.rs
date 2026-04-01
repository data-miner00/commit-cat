use crate::models::ai_provider_catalog::{
    get_ai_provider_catalog as build_ai_provider_catalog, get_ai_provider_catalog_response,
};
use crate::services::storage;
use commit_cat_core::models::ai_provider_catalog::{
    normalize_provider_owned, resolve_model_with_catalog, resolve_reasoning_with_catalog,
    AiProviderCatalogResponse,
};
use commit_cat_core::models::cat_profile::CatPersonalityPreset;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

// Claude API response
#[derive(Deserialize)]
struct ClaudeContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

// OpenAI API response
#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CodexAuthFile {
    #[serde(default)]
    auth_mode: Option<String>,
    #[serde(default)]
    tokens: Option<CodexAuthTokens>,
    #[serde(default, rename = "OPENAI_API_KEY")]
    openai_api_key: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CodexAuthTokens {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexProviderStatus {
    pub available: bool,
    pub authenticated: bool,
    pub connected: bool,
    pub status_message: String,
}

#[tauri::command]
pub fn get_ai_provider_catalog() -> Result<AiProviderCatalogResponse, String> {
    Ok(get_ai_provider_catalog_response())
}

#[tauri::command]
pub async fn chat_with_cat(app: AppHandle, message: String) -> Result<String, String> {
    let data = storage::load(&app)?;
    let provider = normalize_provider_owned(&data.settings.ai_provider);
    let catalog = build_ai_provider_catalog();
    let model = resolve_model_with_catalog(&provider, &data.settings.ai_provider_models, &catalog);
    let reasoning = resolve_reasoning_with_catalog(
        &provider,
        &model,
        &data.settings.ai_provider_reasoning,
        &catalog,
    );

    // Build system prompt (shared)
    let today = &data.today;
    let cat = &data.cat;
    let active_profile = data.active_cat_profile();
    let coding_hours = today.coding_minutes / 60;
    let coding_mins = today.coding_minutes % 60;
    let persona_clause = match active_profile.personality {
        CatPersonalityPreset::Classic => {
            "Personality: balanced, warm, playful, and gently encouraging."
        }
        CatPersonalityPreset::Chill => "Personality: calm, cozy, reassuring, and low-pressure.",
        CatPersonalityPreset::Tsundere => {
            "Personality: a little sharp and teasing, but secretly supportive and affectionate."
        }
        CatPersonalityPreset::Chaotic => {
            "Personality: hyper, playful, dramatic, and full of restless coding energy."
        }
    };

    let system_prompt = format!(
        "You are CommitCat, a small desktop cat companion that lives on a developer's screen. \
         Respond with short, warm messages. Keep it cute but not over the top — \
         no action descriptions like *purrs* or *meows*. \
         Reply in 3-5 sentences max. Always add 1 relevant emoji at the end. \
         Keep total response under 200 characters. \
         Active cat profile: {}. {} \
         User stats: {} commits, {}h{}m coding, Lv.{}.",
        active_profile.name, persona_clause, today.commits, coding_hours, coding_mins, cat.level,
    );

    match provider.as_str() {
        "openai-api" => {
            chat_openai(
                &data.settings.openai_api_key,
                &model,
                reasoning.as_deref(),
                &system_prompt,
                &message,
            )
            .await
        }
        "openai-codex-local" => {
            chat_codex_local(&model, reasoning.as_deref(), &system_prompt, &message).await
        }
        "claude" => {
            chat_claude(
                &data.settings.anthropic_api_key,
                &model,
                &system_prompt,
                &message,
            )
            .await
        }
        _ => Err(format!("Unsupported AI provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn get_codex_provider_status() -> Result<CodexProviderStatus, String> {
    tokio::task::spawn_blocking(run_codex_login_status)
        .await
        .map_err(|e| format!("Failed to check Codex provider status: {}", e))?
}

fn run_codex_login_status() -> Result<CodexProviderStatus, String> {
    let auth_path = codex_auth_path()?;
    let auth_file_exists = auth_path.exists();
    let codex_available = resolve_codex_path().is_ok();

    if auth_file_exists {
        let raw = match fs::read_to_string(&auth_path) {
            Ok(raw) => raw,
            Err(e) => {
                return Ok(CodexProviderStatus {
                    available: codex_available,
                    authenticated: false,
                    connected: false,
                    status_message: format!("Failed to read {}: {}", auth_path.display(), e),
                });
            }
        };
        let auth: CodexAuthFile = match serde_json::from_str(&raw) {
            Ok(auth) => auth,
            Err(e) => {
                return Ok(CodexProviderStatus {
                    available: codex_available,
                    authenticated: false,
                    connected: false,
                    status_message: format!("Failed to parse {}: {}", auth_path.display(), e),
                });
            }
        };

        let has_chatgpt_tokens = auth.tokens.as_ref().is_some_and(|tokens| {
            tokens
                .access_token
                .as_deref()
                .is_some_and(|value| !value.is_empty())
                && tokens
                    .refresh_token
                    .as_deref()
                    .is_some_and(|value| !value.is_empty())
        });
        let has_api_key = auth
            .openai_api_key
            .as_ref()
            .is_some_and(has_configured_value);
        let authenticated = has_chatgpt_tokens || has_api_key;
        let connected = codex_available && authenticated;

        if has_chatgpt_tokens {
            return Ok(CodexProviderStatus {
                available: codex_available,
                authenticated: true,
                connected,
                status_message: if connected {
                    "Connected via Codex auth file.".to_string()
                } else {
                    "Codex auth was found, but the Codex CLI is not installed on this machine."
                        .to_string()
                },
            });
        }

        if has_api_key {
            return Ok(CodexProviderStatus {
                available: codex_available,
                authenticated: true,
                connected,
                status_message: if connected {
                    "Connected via Codex API key auth.".to_string()
                } else {
                    "Codex credentials were found, but the Codex CLI is not installed on this machine."
                        .to_string()
                },
            });
        }

        return Ok(CodexProviderStatus {
            available: codex_available,
            authenticated: false,
            connected: false,
            status_message: format!(
                "{} exists, but no usable Codex credentials were found.",
                auth_path.display()
            ),
        });
    }

    if codex_available {
        Ok(CodexProviderStatus {
            available: true,
            authenticated: false,
            connected: false,
            status_message: format!(
                "Codex CLI is installed, but {} was not found.",
                auth_path.display()
            ),
        })
    } else {
        Ok(CodexProviderStatus {
            available: false,
            authenticated: false,
            connected: false,
            status_message: "Codex CLI is not installed on this machine.".to_string(),
        })
    }
}

async fn chat_codex_local(
    model: &str,
    reasoning: Option<&str>,
    system_prompt: &str,
    message: &str,
) -> Result<String, String> {
    let status = tokio::task::spawn_blocking(run_codex_login_status)
        .await
        .map_err(|e| format!("Failed to check Codex provider status: {}", e))??;

    if !status.available {
        return Err(status.status_message);
    }

    if !status.connected {
        return Err(status.status_message);
    }

    let prompt = format!(
        "{system_prompt}\n\
         You are using the local Codex provider. Reply with the final answer only.\n\
         User message: {message}"
    );
    let model = model.to_string();
    let reasoning = reasoning.map(ToOwned::to_owned);

    tokio::task::spawn_blocking(move || {
        let temp_path = codex_output_path();
        let temp_path_arg = temp_path.to_string_lossy().to_string();
        let codex_path = resolve_codex_path()
            .map_err(|e| format!("Failed to run local Codex provider: {}", e))?;
        let mut command = Command::new(codex_path);
        command.args([
            "exec",
            "--skip-git-repo-check",
            "--sandbox",
            "read-only",
            "--model",
        ]);
        command.arg(&model);
        if let Some(reasoning) = reasoning.as_deref() {
            command.arg("-c");
            command.arg(format!("model_reasoning_effort=\"{}\"", reasoning));
        }
        let output = command
            .arg("--output-last-message")
            .arg(&temp_path_arg)
            .arg(&prompt)
            .output()
            .map_err(|e| format!("Failed to run local Codex provider: {}", e))?;

        if !output.status.success() {
            let _ = fs::remove_file(&temp_path);
            return Err(command_error("Local Codex provider failed", &output));
        }

        let text = fs::read_to_string(&temp_path)
            .map_err(|e| format!("Failed to read Codex response: {}", e))?;
        let _ = fs::remove_file(&temp_path);

        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return Err("Empty response from local Codex provider".into());
        }

        Ok(trimmed)
    })
    .await
    .map_err(|e| format!("Failed to run local Codex provider: {}", e))?
}

fn codex_output_path() -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "commitcat-codex-{}-{}.txt",
        std::process::id(),
        timestamp
    ))
}

fn resolve_codex_path() -> std::io::Result<PathBuf> {
    let direct = Command::new("codex").arg("--version").output();
    if let Ok(output) = direct {
        if output.status.success() {
            return Ok(PathBuf::from("codex"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        for shell in ["zsh", "bash", "sh"] {
            let output = Command::new(shell)
                .args(["-lc", "command -v codex"])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(PathBuf::from(path));
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("where").arg("codex").output();
        if let Ok(output) = output {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Codex CLI executable could not be found",
    ))
}

fn command_error(prefix: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if !stderr.is_empty() {
        format!("{prefix}: {stderr}")
    } else if !stdout.is_empty() {
        format!("{prefix}: {stdout}")
    } else {
        format!("{prefix}: exit status {}", output.status)
    }
}

fn codex_auth_path() -> Result<PathBuf, String> {
    if let Some(codex_home) = std::env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(codex_home).join("auth.json"));
    }

    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or("Could not resolve home directory for Codex auth")?;

    Ok(PathBuf::from(home).join(".codex").join("auth.json"))
}

fn has_configured_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::String(text) => !text.trim().is_empty(),
        serde_json::Value::Object(map) => map.values().any(has_configured_value),
        _ => true,
    }
}

async fn chat_claude(
    api_key: &Option<String>,
    model: &str,
    system_prompt: &str,
    message: &str,
) -> Result<String, String> {
    let api_key = api_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or("No Anthropic API key configured")?;

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 200,
        "system": system_prompt,
        "messages": [{ "role": "user", "content": message }]
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, body_text));
    }

    let parsed: ClaudeResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let text = parsed
        .content
        .into_iter()
        .filter_map(|b| b.text)
        .collect::<Vec<_>>()
        .join("");
    if text.is_empty() {
        return Err("Empty response from API".into());
    }
    Ok(text)
}

async fn chat_openai(
    api_key: &Option<String>,
    model: &str,
    reasoning: Option<&str>,
    system_prompt: &str,
    message: &str,
) -> Result<String, String> {
    let api_key = api_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or("No OpenAI API key configured")?;

    let mut body = serde_json::json!({
        "model": model,
        "max_tokens": 200,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": message }
        ]
    });
    if let Some(reasoning) = reasoning {
        body["reasoning"] = serde_json::json!({ "effort": reasoning });
    }

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, body_text));
    }

    let parsed: OpenAIResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let text = parsed
        .choices
        .into_iter()
        .filter_map(|c| c.message.content)
        .next()
        .unwrap_or_default();
    if text.is_empty() {
        return Err("Empty response from API".into());
    }
    Ok(text)
}
