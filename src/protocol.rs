use serde_json::{json, Map, Value};

use crate::executor::AgentExecutor;
use crate::runtime::AgentRuntime;

pub fn handle_request(runtime: &AgentRuntime, request: Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str);

    match method {
        Some("tools/list") => tools_list(id),
        Some("tools/call") => tools_call(runtime.executor(), id, &request),
        Some("config/ollama/get") => ollama_config_get(runtime, id, &request),
        Some("config/ollama/update") => ollama_config_update(runtime, id, &request),
        _ => error(id, format!("Método não suportado: {}", display(method))),
    }
}

fn ollama_config_get(runtime: &AgentRuntime, id: Value, request: &Value) -> Value {
    if request
        .get("params")
        .is_some_and(|value| !is_empty_object(value))
    {
        return error(id, "config/ollama/get não recebe parâmetros.");
    }
    match runtime.ollama_config().snapshot() {
        Ok(config) => rpc_result(id, config),
        Err(config_error) => error(id, format!("Falha ao ler configuração: {config_error}")),
    }
}

fn ollama_config_update(runtime: &AgentRuntime, id: Value, request: &Value) -> Value {
    let Some(params) = request.get("params").and_then(Value::as_object) else {
        return error(id, "O objeto params é obrigatório.");
    };
    match runtime.ollama_config().update(params) {
        Ok(config) => rpc_result(id, config),
        Err(config_error) => error(id, format!("Configuração recusada: {config_error}")),
    }
}

fn is_empty_object(value: &Value) -> bool {
    value.as_object().is_some_and(Map::is_empty)
}

fn tools_list(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {"tools": [system_info_schema(), list_directory_schema(), create_directory_schema()]}
    })
}

fn system_info_schema() -> Value {
    json!({
        "name": "system.info",
        "description": "Retorna sistema operacional, arquitetura e área de trabalho permitida.",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false},
        "annotations": {"readOnlyHint": true, "destructiveHint": false}
    })
}

fn list_directory_schema() -> Value {
    json!({
        "name": "filesystem.list_directory",
        "description": "Lista até 200 itens de um diretório dentro da área permitida.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string", "description": "Caminho relativo; use '.' para a raiz."}},
            "required": ["path"],
            "additionalProperties": false
        },
        "annotations": {"readOnlyHint": true, "destructiveHint": false}
    })
}

fn create_directory_schema() -> Value {
    json!({
        "name": "filesystem.create_directory",
        "description": "Cria uma pasta dentro da área permitida; não aceita caminhos absolutos ou '..'.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string", "description": "Caminho relativo da nova pasta."}},
            "required": ["path"],
            "additionalProperties": false
        },
        "annotations": {"readOnlyHint": false, "destructiveHint": false, "idempotentHint": true}
    })
}

fn tools_call(executor: &AgentExecutor, id: Value, request: &Value) -> Value {
    let params = request.get("params").and_then(Value::as_object);
    let name = params
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str);
    let arguments = params
        .and_then(|value| value.get("arguments"))
        .and_then(Value::as_object);

    match name {
        Some("system.info") => system_info(executor, id, arguments),
        Some("filesystem.list_directory") => list_directory(executor, id, arguments),
        Some("filesystem.create_directory") => create_directory(executor, id, arguments),
        _ => error(id, format!("Tool desconhecida: {}", display(name))),
    }
}

fn system_info(
    executor: &AgentExecutor,
    id: Value,
    arguments: Option<&Map<String, Value>>,
) -> Value {
    if arguments.is_some_and(|value| !value.is_empty()) {
        return error(id, "A tool system.info não recebe argumentos.");
    }
    json_result(id, executor.system_info())
}

fn list_directory(
    executor: &AgentExecutor,
    id: Value,
    arguments: Option<&Map<String, Value>>,
) -> Value {
    let path = match required_path(arguments) {
        Ok(path) => path,
        Err(message) => return error(id, message),
    };
    operation_result(id, executor.list_directory(path))
}

fn create_directory(
    executor: &AgentExecutor,
    id: Value,
    arguments: Option<&Map<String, Value>>,
) -> Value {
    let path = match required_path(arguments) {
        Ok(path) => path,
        Err(message) => return error(id, message),
    };
    operation_result(id, executor.create_directory(path))
}

fn required_path(arguments: Option<&Map<String, Value>>) -> Result<&str, &'static str> {
    let arguments = arguments.ok_or("O objeto arguments é obrigatório.")?;
    if arguments.len() != 1 || !arguments.contains_key("path") {
        return Err("Apenas o campo 'path' é permitido.");
    }
    arguments
        .get("path")
        .and_then(Value::as_str)
        .ok_or("O campo 'path' deve ser uma string.")
}

fn operation_result(id: Value, result: Result<Value, crate::ExecutorError>) -> Value {
    match result {
        Ok(content) => json_result(id, content),
        Err(executor_error) => error(id, format!("Operação recusada: {executor_error}")),
    }
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn json_result(id: Value, content: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": {"content": [{"type": "json", "json": content}]}})
}

fn error(id: Value, message: impl Into<String>) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32000, "message": message.into()}})
}

fn display(value: Option<&str>) -> &str {
    value.unwrap_or("None")
}
