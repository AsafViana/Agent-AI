# Graph Report - .  (2026-08-03)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 137 nodes · 433 edges · 13 communities (12 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `741c581f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- protocol.rs
- OllamaConfig
- Result
- AgentExecutor
- ConfigError
- ExecutorError
- RuntimeError
- process_line
- config.rs
- .from_environment
- executor.rs
- read_directory_entries
- .fmt

## God Nodes (most connected - your core abstractions)
1. `ConfigError` - 29 edges
2. `ExecutorError` - 25 edges
3. `OllamaConfig` - 21 edges
4. `AgentExecutor` - 17 edges
5. `apply_update()` - 13 edges
6. `OllamaConfigStore` - 12 edges
7. `invalid()` - 12 edges
8. `error()` - 12 edges
9. `AgentRuntime` - 11 edges
10. `update_base_url()` - 10 edges

## Surprising Connections (you probably didn't know these)
- `process_line()` --calls--> `handle_request()`  [INFERRED]
  src/main.rs → src/protocol.rs
- `RuntimeError` --references--> `ConfigError`  [EXTRACTED]
  src/runtime.rs → src/config.rs
- `AgentRuntime` --references--> `OllamaConfigStore`  [EXTRACTED]
  src/runtime.rs → src/config.rs
- `operation_result()` --references--> `ExecutorError`  [EXTRACTED]
  src/protocol.rs → src/executor.rs
- `RuntimeError` --references--> `ExecutorError`  [EXTRACTED]
  src/runtime.rs → src/executor.rs

## Import Cycles
- None detected.

## Communities (13 total, 1 thin omitted)

### Community 0 - "protocol.rs"
Cohesion: 0.23
Nodes (25): create_directory(), create_directory_schema(), display(), error(), handle_request(), is_empty_object(), json_result(), list_directory() (+17 more)

### Community 1 - "OllamaConfig"
Cohesion: 0.27
Nodes (7): Default, RwLock, RwLockReadGuard, RwLockWriteGuard, config_snapshot(), OllamaConfig, OllamaConfigStore

### Community 2 - "Result"
Cohesion: 0.59
Nodes (12): apply_update(), reject_unknown_fields(), required_string(), Map, Result, String, Value, update_api_key() (+4 more)

### Community 3 - "AgentExecutor"
Cohesion: 0.45
Nodes (6): Component, AgentExecutor, append_safe_component(), PathBuf, Result, validate_relative_path()

### Community 4 - "ConfigError"
Cohesion: 0.29
Nodes (9): ConfigError, default_config_path(), require_absolute_config_path(), resolve_config_path(), Display, Error, Formatter, From (+1 more)

### Community 5 - "ExecutorError"
Cohesion: 0.31
Nodes (8): default_workspace_root(), ExecutorError, require_absolute_path(), Display, Error, From, Into, Self

### Community 6 - "RuntimeError"
Cohesion: 0.22
Nodes (6): Display, Error, Formatter, Result, Self, RuntimeError

### Community 7 - "process_line"
Cohesion: 0.42
Nodes (8): Box, main(), parse_error(), process_line(), Error, Result, Value, send()

### Community 8 - "config.rs"
Cohesion: 0.36
Nodes (8): has_valid_authority(), invalid(), is_configured(), is_loopback_url(), normalize_base_url(), OllamaAuthMode, Into, validate_config()

### Community 9 - ".from_environment"
Cohesion: 0.32
Nodes (6): load_config(), persist_config(), read_api_key_from_environment(), Option, Path, Self

### Community 10 - "executor.rs"
Cohesion: 0.50
Nodes (7): entry_name(), inspect_existing_ancestor(), invalid_path(), require_directory(), require_safe_ancestors(), Path, String

### Community 11 - "read_directory_entries"
Cohesion: 0.33
Nodes (5): DirEntry, directory_entry(), read_directory_entries(), Value, Vec

## Knowledge Gaps
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AgentRuntime` connect `protocol.rs` to `OllamaConfig`, `AgentExecutor`, `RuntimeError`, `process_line`?**
  _High betweenness centrality (0.300) - this node is a cross-community bridge._
- **Why does `ExecutorError` connect `ExecutorError` to `protocol.rs`, `AgentExecutor`, `RuntimeError`, `executor.rs`, `read_directory_entries`, `.fmt`?**
  _High betweenness centrality (0.232) - this node is a cross-community bridge._
- **Why does `RuntimeError` connect `RuntimeError` to `ConfigError`, `ExecutorError`?**
  _High betweenness centrality (0.208) - this node is a cross-community bridge._