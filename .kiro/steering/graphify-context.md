---
inclusion: always
---

# Contexto de projeto via Graphify

Para perguntas sobre código, arquitetura, dependências, fluxos ou conteúdo deste projeto, use `graphify-out/` como fonte inicial de contexto persistente.

1. Quando `graphify-out/graph.json` existir, consulte primeiro o grafo com `graphify query "<pergunta>"`.
2. Para investigar relações ou conceitos específicos, use `graphify path "<origem>" "<destino>"` ou `graphify explain "<conceito>"`.
3. Leia `graphify-out/GRAPH_REPORT.md` somente para revisões arquiteturais amplas ou se as consultas ao grafo não fornecerem contexto suficiente.
4. Depois da consulta ao grafo, leia apenas os arquivos-fonte necessários para confirmar fatos, responder à pergunta ou implementar uma alteração.
5. Não trate o grafo como substituto do código-fonte: valide detalhes de implementação nos arquivos relevantes antes de modificá-los.
