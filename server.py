#!/usr/bin/env python3
import sys
import json
import os
import time
from typing import Dict, Any, List

BASE_DIR = "data/notes_storage"

def ensure_base_dir():
    os.makedirs(BASE_DIR, exist_ok=True)

def send(msg: Dict[str, Any]):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

def list_notes() -> List[Dict[str, Any]]:
    ensure_base_dir()
    notes = []
    for fname in os.listdir(BASE_DIR):
        if not fname.endswith(".json"):
            continue
        path = os.path.join(BASE_DIR, fname)
        try:
            with open(path, "r", encoding="utf-8") as f:
                data = json.load(f)
            notes.append({
                "id": data.get("id"),
                "title": data.get("title"),
                "created_at": data.get("created_at"),
            })
        except Exception:
            # ignora arquivo corrompido
            continue
    return notes

def add_note(title: str, content: str) -> Dict[str, Any]:
    ensure_base_dir()
    note_id = str(int(time.time() * 1000))
    note = {
        "id": note_id,
        "title": title,
        "content": content,
        "created_at": time.time(),
    }
    path = os.path.join(BASE_DIR, f"{note_id}.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(note, f, ensure_ascii=False)
    return note

def search_notes(query: str) -> List[Dict[str, Any]]:
    ensure_base_dir()
    results = []
    for fname in os.listdir(BASE_DIR):
        if not fname.endswith(".json"):
            continue
        path = os.path.join(BASE_DIR, fname)
        try:
            with open(path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except Exception:
            continue
        text = (data.get("title", "") + " " + data.get("content", "")).lower()
        if query.lower() in text:
            results.append({
                "id": data.get("id"),
                "title": data.get("title"),
                "snippet": data.get("content", "")[:80],
            })
    return results

def handle_request(req: Dict[str, Any]) -> Dict[str, Any]:
    method = req.get("method")
    req_id = req.get("id")

    def error(message: str):
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {"code": -32000, "message": message},
        }

    # tools/list
    if method == "tools/list":
        result = {
            "tools": [
                {
                    "name": "notes.add",
                    "description": "Adiciona uma nova nota.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "content": {"type": "string"},
                        },
                        "required": ["title", "content"],
                        "additionalProperties": False,
                    },
                },
                {
                    "name": "notes.list",
                    "description": "Lista notas existentes.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "additionalProperties": False,
                    },
                },
                {
                    "name": "notes.search",
                    "description": "Busca notas contendo um termo.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {"type": "string"},
                        },
                        "required": ["query"],
                        "additionalProperties": False,
                    },
                },
            ]
        }
        return {"jsonrpc": "2.0", "id": req_id, "result": result}

    # tools/call
    if method == "tools/call":
        params = req.get("params") or {}
        name = params.get("name")
        arguments = params.get("arguments") or {}

        if name == "notes.add":
            title = arguments.get("title")
            content = arguments.get("content")
            if not isinstance(title, str) or not isinstance(content, str):
                return error("Campos 'title' e 'content' são obrigatórios e devem ser strings.")
            note = add_note(title, content)
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": f"Nota criada com id={note['id']} e título='{note['title']}'",
                        }
                    ]
                },
            }

        if name == "notes.list":
            notes = list_notes()
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "json",
                            "json": notes,
                        }
                    ]
                },
            }

        if name == "notes.search":
            query = arguments.get("query")
            if not isinstance(query, str):
                return error("Campo 'query' é obrigatório e deve ser string.")
            results = search_notes(query)
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "json",
                            "json": results,
                        }
                    ]
                },
            }

        return error(f"Tool desconhecida: {name}")

    return error(f"Método não suportado: {method}")

def main():
    ensure_base_dir()
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as e:
            send({
                "jsonrpc": "2.0",
                "id": None,
                "error": {"code": -32700, "message": f"JSON inválido: {e}"},
            })
            continue

        resp = handle_request(req)
        send(resp)

if __name__ == "__main__":
    main()
