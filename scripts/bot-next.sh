#!/usr/bin/env bash
# Painel do "bot programador": mostra o que a IA/dev deve fazer agora.
# Uso: ./scripts/bot-next.sh   (requer gh autenticado)
set -euo pipefail

echo "==================== MVCAP2P · Painel do Bot ===================="

echo
echo "🧪 Rodadas de teste AGUARDANDO sign-off dos testers:"
gh issue list --label test-round --state open \
  --json number,title,url -q '.[] | "  #\(.number)  \(.title)\n         \(.url)"' || true

echo
echo "🔧 Em progresso (tasks abertas com label prio:P0/P1, exceto test-round):"
gh issue list --state open --label epic \
  --json number,title,labels,url \
  -q '.[] | select([.labels[].name] | index("test-round") | not)
        | "  #\(.number)  \(.title)   [\([.labels[].name] | join(", ")) ]"' || true

echo
echo "▶️  PRÓXIMA tarefa sugerida (maior prioridade não bloqueada):"
gh issue list --state open --label epic --json number,title,labels,url \
  -q 'sort_by([.labels[].name] | map(select(startswith("prio:"))) | .[0]) | .[0]
      | if . == null then "  (nenhuma pendente 🎉)"
        else "  #\(.number)  \(.title)\n         \(.url)" end' || true

echo
echo "Dica: trabalhe a próxima tarefa, faça PR com 'Closes #<n>' e publique um"
echo "Release para disparar a Rodada de Testes automaticamente."
echo "================================================================"
