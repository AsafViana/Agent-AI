//! Executor local seguro exposto ao AetherCore por MCP/JSON-RPC sobre STDIO.

mod config;
mod executor;
mod protocol;
mod runtime;

pub use config::{ConfigError, OllamaAuthMode, OllamaConfig, OllamaConfigStore};
pub use executor::{AgentExecutor, ExecutorError};
pub use protocol::handle_request;
pub use runtime::{AgentRuntime, RuntimeError};
