use std::error::Error;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::{json, Value};

const WORKSPACE_ENV: &str = "AETHERCORE_AGENT_WORKSPACE";
const MAX_DIRECTORY_ENTRIES: usize = 200;
const MAX_RELATIVE_PATH_LENGTH: usize = 240;

#[derive(Debug)]
pub enum ExecutorError {
    Configuration(String),
    InvalidPath(String),
    Io(std::io::Error),
}

impl Display for ExecutorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => write!(formatter, "configuração inválida: {message}"),
            Self::InvalidPath(message) => write!(formatter, "caminho inválido: {message}"),
            Self::Io(error) => write!(formatter, "erro de I/O: {error}"),
        }
    }
}

impl Error for ExecutorError {}

impl From<std::io::Error> for ExecutorError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug)]
pub struct AgentExecutor {
    workspace_root: PathBuf,
}

impl AgentExecutor {
    pub fn from_environment() -> Result<Self, ExecutorError> {
        let root = match std::env::var_os(WORKSPACE_ENV) {
            Some(value) => require_absolute_path(PathBuf::from(value))?,
            None => default_workspace_root(),
        };
        Self::new(root)
    }

    pub fn new(workspace_root: impl Into<PathBuf>) -> Result<Self, ExecutorError> {
        let workspace_root = workspace_root.into();
        fs::create_dir_all(&workspace_root)?;
        let workspace_root = workspace_root.canonicalize()?;
        Ok(Self { workspace_root })
    }

    pub fn system_info(&self) -> Value {
        json!({
            "os": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
            "workspaceRoot": self.workspace_root.to_string_lossy()
        })
    }

    pub fn list_directory(&self, relative_path: &str) -> Result<Value, ExecutorError> {
        let directory = self.resolve_existing(relative_path)?;
        require_directory(&directory)?;
        let mut entries = read_directory_entries(&directory)?;
        entries.sort_by_key(entry_name);
        let is_truncated = entries.len() > MAX_DIRECTORY_ENTRIES;
        entries.truncate(MAX_DIRECTORY_ENTRIES);
        Ok(json!({"path": relative_path, "entries": entries, "isTruncated": is_truncated}))
    }

    pub fn create_directory(&self, relative_path: &str) -> Result<Value, ExecutorError> {
        let destination = self.resolve_destination(relative_path)?;
        if destination == self.workspace_root {
            return Err(invalid_path("informe uma subpasta da área permitida"));
        }
        fs::create_dir_all(&destination)?;
        self.require_within_workspace(&destination.canonicalize()?)?;
        Ok(json!({"path": relative_path, "created": true}))
    }

    fn resolve_existing(&self, relative_path: &str) -> Result<PathBuf, ExecutorError> {
        let candidate = self
            .workspace_root
            .join(validate_relative_path(relative_path)?);
        let resolved = candidate.canonicalize()?;
        self.require_within_workspace(&resolved)?;
        Ok(resolved)
    }

    fn resolve_destination(&self, relative_path: &str) -> Result<PathBuf, ExecutorError> {
        let destination = self
            .workspace_root
            .join(validate_relative_path(relative_path)?);
        require_safe_ancestors(&self.workspace_root, &destination)?;
        Ok(destination)
    }

    fn require_within_workspace(&self, path: &Path) -> Result<(), ExecutorError> {
        if path.starts_with(&self.workspace_root) {
            return Ok(());
        }
        Err(invalid_path("o caminho sai da área permitida"))
    }
}

fn default_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/workspace")
}

fn require_absolute_path(path: PathBuf) -> Result<PathBuf, ExecutorError> {
    if path.is_absolute() {
        return Ok(path);
    }
    Err(ExecutorError::Configuration(format!(
        "{WORKSPACE_ENV} deve conter um caminho absoluto"
    )))
}

fn validate_relative_path(value: &str) -> Result<PathBuf, ExecutorError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_RELATIVE_PATH_LENGTH {
        return Err(invalid_path(
            "use um caminho relativo não vazio com até 240 caracteres",
        ));
    }
    let mut clean = PathBuf::new();
    for component in Path::new(value).components() {
        append_safe_component(&mut clean, component)?;
    }
    Ok(if clean.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        clean
    })
}

fn append_safe_component(
    path: &mut PathBuf,
    component: Component<'_>,
) -> Result<(), ExecutorError> {
    match component {
        Component::Normal(value) => path.push(value),
        Component::CurDir => {}
        _ => return Err(invalid_path("caminhos absolutos e '..' não são permitidos")),
    }
    Ok(())
}

fn require_safe_ancestors(root: &Path, destination: &Path) -> Result<(), ExecutorError> {
    let mut current = root.to_path_buf();
    for component in destination
        .strip_prefix(root)
        .unwrap_or(destination)
        .components()
    {
        current.push(component);
        inspect_existing_ancestor(root, &current)?;
    }
    Ok(())
}

fn inspect_existing_ancestor(root: &Path, path: &Path) -> Result<(), ExecutorError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(invalid_path("links simbólicos não são permitidos"))
        }
        Ok(_) if path.canonicalize()?.starts_with(root) => Ok(()),
        Ok(_) => Err(invalid_path("o caminho sai da área permitida")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn require_directory(path: &Path) -> Result<(), ExecutorError> {
    if path.is_dir() {
        return Ok(());
    }
    Err(invalid_path("o destino não é um diretório"))
}

fn read_directory_entries(path: &Path) -> Result<Vec<Value>, ExecutorError> {
    fs::read_dir(path)?
        .map(|entry| entry.map_err(ExecutorError::from).and_then(directory_entry))
        .collect()
}

fn directory_entry(entry: fs::DirEntry) -> Result<Value, ExecutorError> {
    let file_type = entry.file_type()?;
    let kind = if file_type.is_symlink() {
        "symlink"
    } else if file_type.is_dir() {
        "directory"
    } else {
        "file"
    };
    Ok(json!({"name": entry.file_name().to_string_lossy(), "type": kind}))
}

fn entry_name(entry: &Value) -> String {
    entry
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase()
}

fn invalid_path(message: impl Into<String>) -> ExecutorError {
    ExecutorError::InvalidPath(message.into())
}
