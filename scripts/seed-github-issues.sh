#!/usr/bin/env bash
# Seed do backlog do MVCAP2P no GitHub Issues + GitHub Projects.
# Transforma o Backlog de Épicos da Especificação Técnica Mestre (SSOT) em Issues
# com checklists de Critérios de Aceite e campos de Prioridade/Esforço.
#
# Pré-requisitos:
#   - gh CLI instalado e autenticado:  gh auth login
#   - scopes: repo, project
#
# Uso:
#   ./scripts/seed-github-issues.sh              # cria labels + issues
#   PROJECT_NUMBER=1 ./scripts/seed-github-issues.sh   # tambem adiciona ao Project #1
set -euo pipefail

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
echo ">> Repositório: $REPO"

# --- Labels (Prioridade / Esforço / Área) -----------------------------------
mklabel() { gh label create "$1" --color "$2" --description "$3" --force >/dev/null 2>&1 || true; }

mklabel "prio:P0"        "b60205" "Bloqueante / crítico"
mklabel "prio:P1"        "d93f0b" "Alta prioridade"
mklabel "prio:P2"        "fbca04" "Média prioridade"
mklabel "effort:1"       "c2e0c6" "Story Points: 1"
mklabel "effort:3"       "bfd4f2" "Story Points: 3"
mklabel "effort:5"       "5319e7" "Story Points: 5"
mklabel "effort:8"       "0e8a16" "Story Points: 8"
mklabel "area:infra"     "cccccc" "Infraestrutura / build"
mklabel "area:rede"      "1d76db" "Rede / sincronização"
mklabel "area:fisica"    "0052cc" "Física / determinismo"
mklabel "area:ui"        "d4c5f9" "Interface 2.0"
mklabel "area:qa"        "006b75" "QA / performance"
mklabel "epic"           "5319e7" "Épico da SSOT"

# --- Helper: cria issue se ainda não existir (por título) -------------------
mkissue() {
  local title="$1" body="$2"; shift 2
  if gh issue list --search "in:title \"$title\"" --state all --json title -q '.[].title' | grep -qxF "$title"; then
    echo ">> já existe: $title"; return 0
  fi
  local args=(); for l in "$@"; do args+=(--label "$l"); done
  gh issue create --title "$title" --body "$body" "${args[@]}"
}

# --- Épico 1: Infra ---------------------------------------------------------
mkissue "[Infra] Configuração de Workspace e Cache" \
"## Ação
Implementar estrutura de membros \`server\`/\`client\` e habilitar \`sccache\` via \`.cargo/config.toml\`.

## Critérios de Aceite
- [ ] \`cargo build --workspace\` compila ambos os membros simultaneamente
- [ ] \`sccache --show-stats\` confirma cache hits após a 2ª compilação limpa
- [ ] \`SCCACHE_BASEDIRS\` e \`SCCACHE_IGNORE_SERVER_IO_ERROR=1\` documentados no README/AGENTS.md

## SSOT
Ref.: Especificação Técnica Mestre §2 e §8.1 (\`docs/content/docs/spec\`)." \
  "epic" "area:infra" "prio:P0" "effort:3"

# --- Épico 2: Rede ----------------------------------------------------------
mkissue "[Rede] Core de Identidade SpacetimeDB" \
"## Ação
Criar tabela \`Player\` com chave primária \`Identity\` e implementar lógica de token persistente.

## Critérios de Aceite
- [ ] Peer identificado por \`Identity\` de 256-bit derivada de token local
- [ ] Cliente reconhece o mesmo \`username\` após reinício da aplicação
- [ ] Validado via query SQL no servidor
- [ ] Toda mutação via Reducer (proibido \`rand\` padrão; usar \`ctx.rng()\`)

## SSOT
Ref.: Especificação Técnica Mestre §3 e §8.2." \
  "epic" "area:rede" "prio:P1" "effort:8"

# --- Épico 3: Física --------------------------------------------------------
mkissue "[Física] Setup Rapier Determinístico" \
"## Ação
Configurar features de determinismo e implementar validação de checksum de estado.

## Critérios de Aceite
- [ ] Feature \`enhanced-determinism\` ativa; SIMD desativado se causar divergência
- [ ] Feature \`serde-serialize\` para snapshots de estado físico
- [ ] 1000 frames sem desync entre x86 (Linux) e ARM (validado por \`SyncTestSession\`)
- [ ] Sem drift bit-a-bit entre arquiteturas

## SSOT
Ref.: Especificação Técnica Mestre §4 e §8.3. Ver ADR-010." \
  "epic" "area:fisica" "prio:P1" "effort:8"

# --- Épico 4: UI ------------------------------------------------------------
mkissue "[UI] Framework de Interface ECS" \
"## Ação
Implementar Top Inspector assíncrono exibindo FPS e RTT.

## Critérios de Aceite
- [ ] UI como entidades Bevy (proibido Immediate Mode UI exclusivo)
- [ ] Mantém 60 FPS estáveis mesmo com queda de perf na thread de física (carga artificial)
- [ ] Floating Menu como sub-árvore de entidades (persiste durante rollbacks)
- [ ] Dual stick com zonas mortas configuráveis

## SSOT
Ref.: Especificação Técnica Mestre §5 e §8.4. Ver ADR-009." \
  "epic" "area:ui" "prio:P2" "effort:5"

# --- Issue bootstrap de QA (Mandato de Gerência) ----------------------------
mkissue "[QA] Configuração de Ambiente de QA (Raspberry Pi 3)" \
"## Ação
Configurar o Raspberry Pi 3 como ambiente headless de QA, idêntico ao de produção,
usando os scripts de \`scripts/\` (ex.: \`setup-pi.sh\`, \`deploy-farm.sh\`).

## Critérios de Aceite (KPIs — Gate de Commit)
- [ ] FPS > 30 estáveis em cena de estresse (10 jogadores)
- [ ] RAM < 512MB (aplicação total)
- [ ] Latência de processamento de Reducer < 50ms
- [ ] Primeiro relatório postado como comentário (FPS / Latência P2P / Memória / Dispositivo)

## Relato obrigatório por commit
\`\`\`
Dispositivo testado: (ex.: Samsung J7 - Android antigo)
FPS médio:
Latência P2P (via RPi 3):
Consumo de memória:
\`\`\`

## SSOT
Ref.: Especificação Técnica Mestre §7." \
  "area:qa" "prio:P0" "effort:3"

# --- Opcional: adicionar ao GitHub Project -----------------------------------
if [[ -n "${PROJECT_NUMBER:-}" ]]; then
  OWNER="${REPO%%/*}"
  echo ">> Adicionando issues ao Project #$PROJECT_NUMBER de $OWNER ..."
  gh issue list --state open --json url -q '.[].url' | while read -r url; do
    gh project item-add "$PROJECT_NUMBER" --owner "$OWNER" --url "$url" >/dev/null 2>&1 \
      && echo "   + $url" || echo "   ! falhou: $url"
  done
fi

echo ">> Concluído."
