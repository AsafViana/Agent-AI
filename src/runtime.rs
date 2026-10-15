use crate::config::{ConfigError, OllamaConfigStore};
use crate::executor::{AgentExecutor, ExecutorError};

#[derive(Debug)]
pub enum RuntimeError {
    Config(ConfigError),
    Executor(ExecutorError),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "{error}"),
            Self::Executor(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub struct AgentRuntime {
    executor: AgentExecutor,
    ollama_config: OllamaConfigStore,
}

impl AgentRuntime {
    pub fn from_environment() -> Result<Self, RuntimeError> {
        let executor = AgentExecutor::from_environment().map_err(RuntimeError::Executor)?;
        let ollama_config = OllamaConfigStore::from_environment().map_err(RuntimeError::Config)?;
        Ok(Self {
            executor,
            ollama_config,
        })
    }

    pub fn executor(&self) -> &AgentExecutor {
        &self.executor
    }

    pub fn ollama_config(&self) -> &OllamaConfigStore {
        &self.ollama_config
    }
}
