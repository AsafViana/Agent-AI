use std::io::{self, BufRead, Write};

use agent_ai::{handle_request, AgentRuntime};
use serde_json::{json, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = AgentRuntime::from_environment()?;

    for line in io::stdin().lock().lines() {
        process_line(&runtime, &line?)?;
    }

    Ok(())
}

fn process_line(runtime: &AgentRuntime, line: &str) -> io::Result<()> {
    let input = line.trim();
    if input.is_empty() {
        return Ok(());
    }

    let response = match serde_json::from_str(input) {
        Ok(request) => handle_request(runtime, request),
        Err(error) => parse_error(error),
    };
    send(&response)
}

fn parse_error(error: serde_json::Error) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {"code": -32700, "message": format!("JSON inválido: {error}")}
    })
}

fn send(message: &Value) -> io::Result<()> {
    let mut output = io::stdout().lock();
    writeln!(output, "{}", serde_json::to_string(message)?)?;
    output.flush()
}
