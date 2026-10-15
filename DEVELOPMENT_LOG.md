# Diário de desenvolvimento — Agent-AI

## 2026-08-03 — Início do executor local

### Objetivo

Criar um agente local que receba chamadas estruturadas produzidas por um modelo Ollama remoto e execute somente operações autorizadas no computador.

### Arquitetura decidida

- O AetherCore Host mantém a conversa e chama a API do Ollama.
- O Ollama propõe ações por tool calling; não envia comandos misturados ao texto.
- O Host aplica autorização e chama este patch via MCP/JSON-RPC sobre STDIO.
- O patch valida novamente os argumentos e executa a operação local.
- Resultados retornam ao Host e depois ao Ollama para compor a resposta final.

### Decisões de segurança

- Nenhuma ferramenta de shell arbitrário nesta fase.
- Operações de arquivo ficam confinadas a uma raiz explícita.
- Caminhos absolutos, componentes `..` e links simbólicos são recusados.
- A raiz de produção deve ser informada por `AETHERCORE_AGENT_WORKSPACE`.
- O `stdout` fica reservado exclusivamente às mensagens do protocolo.

### Implementado

- Remoção do domínio temporário de notas.
- Renomeação do pacote/binário para `agent_ai`/`agent-ai`.
- Tool somente leitura `system.info`.
- Tool somente leitura `filesystem.list_directory`.
- Tool idempotente `filesystem.create_directory`.
- Anotações MCP indicando leitura, mutação e idempotência.
- Limite de 200 itens por listagem e 240 caracteres por caminho relativo.

### Próximos incrementos

1. Implementar handshake MCP (`initialize` e notifications) conforme o contrato do Host.
2. Adicionar IDs de ação, auditoria e política de aprovação no AetherCore Host.
3. Integrar o Host com `/api/chat` do Ollama e o loop de tool calling.
4. Adicionar leitura e escrita de arquivos com limites, confirmação e trilha de auditoria.
5. Projetar os estados de turno: pensando, aguardando aprovação, executando e respondendo.

### Validação do incremento

- `cargo fmt --all`: concluído.
- `cargo check --locked`: concluído.
- `cargo clippy --locked -- -D warnings`: concluído sem avisos.
- Smoke test de `tools/list`, `system.info`, criação e listagem de pasta: concluído.
- Tentativa com `../blocked`: recusada conforme a política de isolamento.

## 2026-08-03 — Configuração Ollama em runtime

### Implementado

- Estado concorrente de configuração por `RwLock`.
- Métodos administrativos `config/ollama/get` e `config/ollama/update` fora de `tools/list`.
- Atualização sem recompilação de URL, modelo, autenticação e timeout.
- Persistência não secreta em `data/config/ollama.json`.
- Chave Bearer apenas em memória, com carregamento opcional de `AETHERCORE_OLLAMA_API_KEY` ao iniciar.
- Rejeição de URL HTTP remota; HTTP permanece permitido somente para loopback local.

### Validação

- `cargo check --locked`: concluído.
- `cargo clippy --locked -- -D warnings`: concluído sem avisos.
- Consulta e atualização de runtime: concluídas.
- Chave verificada como ausente das respostas e do JSON persistido.
- Tentativa de URL HTTP remota: recusada.
