use std::error::Error;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

const CONFIG_PATH_ENV: &str = "AETHERCORE_AGENT_CONFIG";
const API_KEY_ENV: &str = "AETHERCORE_OLLAMA_API_KEY";
const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const MAX_TIMEOUT_SECONDS: u64 = 600;
const MAX_API_KEY_LENGTH: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OllamaAuthMode {
    None,
    Bearer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub auth_mode: OllamaAuthMode,
    pub timeout_seconds: u64,
    #[serde(skip)]
    api_key: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:11434".to_owned(),
            model: String::new(),
            auth_mode: OllamaAuthMode::None,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            api_key: None,
        }
    }
}
#[derive(Debug)]
pub enum ConfigError {
    Invalid(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Lock,
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "{message}"),
            Self::Io(error) => write!(formatter, "erro de I/O: {error}"),
            Self::Json(error) => write!(formatter, "erro de JSON: {error}"),
            Self::Lock => write!(formatter, "estado de configuração indisponível"),
        }
    }
}

impl Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug)]
pub struct OllamaConfigStore {
    config_path: PathBuf,
    state: RwLock<OllamaConfig>,
}

impl OllamaConfigStore {
    pub fn from_environment() -> Result<Self, ConfigError> {
        let config_path = resolve_config_path()?;
        let mut config = load_config(&config_path)?;
        config.api_key = read_api_key_from_environment();
        validate_config(&config)?;
        persist_config(&config_path, &config)?;
        Ok(Self {
            config_path,
            state: RwLock::new(config),
        })
    }

    pub fn snapshot(&self) -> Result<Value, ConfigError> {
        let config = self.read()?;
        Ok(config_snapshot(&config))
    }

    pub fn update(&self, params: &Map<String, Value>) -> Result<Value, ConfigError> {
        reject_unknown_fields(params)?;
        let mut next = self.read()?.clone();
        apply_update(&mut next, params)?;
        validate_config(&next)?;
        persist_config(&self.config_path, &next)?;
        *self.write()? = next;
        self.snapshot()
    }

    pub fn current(&self) -> Result<OllamaConfig, ConfigError> {
        Ok(self.read()?.clone())
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, OllamaConfig>, ConfigError> {
        self.state.read().map_err(|_| ConfigError::Lock)
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, OllamaConfig>, ConfigError> {
        self.state.write().map_err(|_| ConfigError::Lock)
    }
}
fn resolve_config_path() -> Result<PathBuf, ConfigError> {
    match std::env::var_os(CONFIG_PATH_ENV) {
        Some(value) => require_absolute_config_path(PathBuf::from(value)),
        None => Ok(default_config_path()),
    }
}

fn require_absolute_config_path(path: PathBuf) -> Result<PathBuf, ConfigError> {
    if path.is_absolute() {
        return Ok(path);
    }
    Err(invalid(format!(
        "{CONFIG_PATH_ENV} deve conter um caminho absoluto"
    )))
}

fn default_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/config/ollama.json")
}

fn load_config(path: &Path) -> Result<OllamaConfig, ConfigError> {
    if !path.exists() {
        return Ok(OllamaConfig::default());
    }
    let content = fs::read(path)?;
    Ok(serde_json::from_slice(&content)?)
}

fn persist_config(path: &Path, config: &OllamaConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(config)?)?;
    Ok(())
}

fn read_api_key_from_environment() -> Option<String> {
    std::env::var(API_KEY_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn reject_unknown_fields(params: &Map<String, Value>) -> Result<(), ConfigError> {
    const ALLOWED: [&str; 5] = ["baseUrl", "model", "authMode", "timeoutSeconds", "apiKey"];
    if let Some(field) = params
        .keys()
        .find(|field| !ALLOWED.contains(&field.as_str()))
    {
        return Err(invalid(format!("parâmetro desconhecido: {field}")));
    }
    Ok(())
}

fn apply_update(config: &mut OllamaConfig, params: &Map<String, Value>) -> Result<(), ConfigError> {
    update_base_url(config, params)?;
    update_model(config, params)?;
    update_auth_mode(config, params)?;
    update_timeout(config, params)?;
    update_api_key(config, params)
}

fn update_base_url(
    config: &mut OllamaConfig,
    params: &Map<String, Value>,
) -> Result<(), ConfigError> {
    if let Some(value) = params.get("baseUrl") {
        config.base_url = normalize_base_url(required_string(value, "baseUrl")?)?;
    }
    Ok(())
}

fn update_model(config: &mut OllamaConfig, params: &Map<String, Value>) -> Result<(), ConfigError> {
    if let Some(value) = params.get("model") {
        config.model = required_string(value, "model")?.trim().to_owned();
    }
    Ok(())
}
fn update_auth_mode(
    config: &mut OllamaConfig,
    params: &Map<String, Value>,
) -> Result<(), ConfigError> {
    let Some(value) = params.get("authMode") else {
        return Ok(());
    };
    config.auth_mode = match required_string(value, "authMode")? {
        "none" => OllamaAuthMode::None,
        "bearer" => OllamaAuthMode::Bearer,
        _ => return Err(invalid("authMode deve ser 'none' ou 'bearer'")),
    };
    Ok(())
}

fn update_timeout(
    config: &mut OllamaConfig,
    params: &Map<String, Value>,
) -> Result<(), ConfigError> {
    let Some(value) = params.get("timeoutSeconds") else {
        return Ok(());
    };
    let timeout = value
        .as_u64()
        .ok_or_else(|| invalid("timeoutSeconds deve ser inteiro"))?;
    if !(1..=MAX_TIMEOUT_SECONDS).contains(&timeout) {
        return Err(invalid("timeoutSeconds deve estar entre 1 e 600"));
    }
    config.timeout_seconds = timeout;
    Ok(())
}

fn update_api_key(
    config: &mut OllamaConfig,
    params: &Map<String, Value>,
) -> Result<(), ConfigError> {
    let Some(value) = params.get("apiKey") else {
        return Ok(());
    };
    if value.is_null() {
        config.api_key = None;
        return Ok(());
    }
    let key = required_string(value, "apiKey")?.trim();
    if key.is_empty() || key.chars().count() > MAX_API_KEY_LENGTH {
        return Err(invalid("apiKey deve conter entre 1 e 4096 caracteres"));
    }
    config.api_key = Some(key.to_owned());
    Ok(())
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, ConfigError> {
    value
        .as_str()
        .ok_or_else(|| invalid(format!("{field} deve ser uma string")))
}

fn validate_config(config: &OllamaConfig) -> Result<(), ConfigError> {
    normalize_base_url(&config.base_url)?;
    if config.model.chars().count() > 200 {
        return Err(invalid("model deve conter no máximo 200 caracteres"));
    }
    if !(1..=MAX_TIMEOUT_SECONDS).contains(&config.timeout_seconds) {
        return Err(invalid("timeoutSeconds deve estar entre 1 e 600"));
    }
    Ok(())
}

fn normalize_base_url(value: &str) -> Result<String, ConfigError> {
    let value = value.trim().trim_end_matches('/');
    if value.starts_with("https://") && has_valid_authority(value, "https://") {
        return Ok(value.to_owned());
    }
    if value.starts_with("http://") && is_loopback_url(value) {
        return Ok(value.to_owned());
    }
    Err(invalid(
        "baseUrl deve usar HTTPS; HTTP é permitido apenas para localhost",
    ))
}

fn has_valid_authority(value: &str, scheme: &str) -> bool {
    let authority = value
        .trim_start_matches(scheme)
        .split('/')
        .next()
        .unwrap_or_default();
    !authority.is_empty() && !authority.contains('@') && !authority.chars().any(char::is_whitespace)
}

fn is_loopback_url(value: &str) -> bool {
    let authority = value
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or_default();
    authority == "localhost"
        || authority.starts_with("localhost:")
        || authority == "127.0.0.1"
        || authority.starts_with("127.0.0.1:")
        || authority == "[::1]"
        || authority.starts_with("[::1]:")
}
fn config_snapshot(config: &OllamaConfig) -> Value {
    let auth_mode = match config.auth_mode {
        OllamaAuthMode::None => "none",
        OllamaAuthMode::Bearer => "bearer",
    };
    json!({
        "baseUrl": config.base_url,
        "model": config.model,
        "authMode": auth_mode,
        "timeoutSeconds": config.timeout_seconds,
        "hasApiKey": config.api_key.is_some(),
        "isConfigured": is_configured(config),
        "apiKeyPersistence": "memory-or-environment"
    })
}

fn is_configured(config: &OllamaConfig) -> bool {
    if config.model.trim().is_empty() {
        return false;
    }
    !matches!(config.auth_mode, OllamaAuthMode::Bearer) || config.api_key.is_some()
}

fn invalid(message: impl Into<String>) -> ConfigError {
    ConfigError::Invalid(message.into())
}
