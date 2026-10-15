# Agent-AI

Patch executor local do AetherCore. O processo usa transporte `stdio`, recebe JSON-RPC por linha e expõe ferramentas MCP restritas para execução no computador.

## Ferramentas iniciais

- `system.info`: retorna sistema, arquitetura e raiz permitida.
- `filesystem.list_directory`: lista até 200 itens dentro da raiz permitida.
- `filesystem.create_directory`: cria uma pasta relativa à raiz permitida.

Caminhos absolutos, `..` e travessia por links simbólicos são recusados. Não existe ferramenta de shell arbitrário.

## Área de trabalho permitida

Por padrão, o executor usa `data/workspace`, ancorado no diretório do projeto. Em produção, configure um caminho absoluto:

```powershell
$env:AETHERCORE_AGENT_WORKSPACE = "C:\AetherCore\workspace"
```

## Configuração do Ollama em runtime

A configuração é administrativa e não aparece em `tools/list`; portanto, o modelo não pode alterá-la usando tool calling. O AetherCore Host deve encaminhar estes métodos somente a partir da tela ou API administrativa autenticada.

### Consultar a configuração

```json
{"jsonrpc":"2.0","id":1,"method":"config/ollama/get"}
```

### Atualizar a configuração

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "config/ollama/update",
  "params": {
    "baseUrl": "https://ollama.seudominio.com",
    "model": "qwen3:8b",
    "authMode": "bearer",
    "timeoutSeconds": 120,
    "apiKey": "<chave-secreta>"
  }
}
```

Parâmetros aceitos: `baseUrl`, `model`, `authMode` (`none` ou `bearer`), `timeoutSeconds` (1–600) e `apiKey`. URLs remotas exigem HTTPS; HTTP só é permitido em `localhost`, `127.0.0.1` ou `::1`.

`baseUrl`, `model`, `authMode` e `timeoutSeconds` são salvos automaticamente em `data/config/ollama.json`. A `apiKey` é mantida somente na memória e nunca aparece na resposta ou no arquivo. Após reiniciar o processo, reenvie-a pelo método administrativo ou forneça-a no ambiente do processo:

```powershell
$env:AETHERCORE_OLLAMA_API_KEY = "sua-chave"
```

Para mudar o local do arquivo não secreto de configuração, use `AETHERCORE_AGENT_CONFIG` com um caminho absoluto.

## Estrutura

- `src/executor.rs`: política de caminhos e operações locais.
- `src/protocol.rs`: catálogo e despacho MCP/JSON-RPC.
- `src/main.rs`: servidor STDIO.
- `DEVELOPMENT_LOG.md`: decisões e evolução do projeto.

## Executar

```powershell
cargo run --quiet --bin agent-ai
```

Cada requisição ocupa uma linha no `stdin`; cada resposta ocupa uma linha no `stdout`. O `stdout` é reservado ao protocolo.

Exemplo para listar ferramentas:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
```

Para produção:

```powershell
cargo build --release
```

Configure o Host para iniciar `target\release\agent-ai.exe` com `transport: stdio`. O Host deve executar o entrypoint a partir do diretório do patch ou usar um caminho absoluto para o binário.
