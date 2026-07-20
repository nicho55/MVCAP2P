#!/usr/bin/env bash
# setup-pi.sh — configura um Raspberry Pi como NÓ DE TESTE (test farm) do
# Tabletop P2P: ADB para os celulares fracos + GitHub Actions self-hosted
# runner (label android-farm) que roda o job de deploy/medição.
#
# Uso na tela do Pi (como usuário NORMAL, não root):
#   curl -fsSL https://raw.githubusercontent.com/nicho55/MVCAP2P/main/scripts/setup-pi.sh | bash
#
# Opcional: export RUNNER_TOKEN=xxxx antes (senão o script pergunta).
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/nicho55/MVCAP2P}"
RUNNER_LABELS="${RUNNER_LABELS:-android-farm,rpi}"
RUNNER_NAME="${RUNNER_NAME:-pi-$(hostname)}"

log(){ printf '\n\033[1;36m▸ %s\033[0m\n' "$*"; }
die(){ printf '\n\033[1;31m✗ %s\033[0m\n' "$*"; exit 1; }

[ "$(id -u)" != "0" ] || die "Rode como usuário normal (SEM sudo). O script usa sudo só onde precisa."

# 1. Pacotes essenciais ------------------------------------------------------
log "Instalando adb, git, curl, jq..."
sudo apt-get update -y
sudo apt-get install -y adb git curl jq

# 2. Regras udev para acesso USB aos celulares sem root ----------------------
log "Configurando udev (ADB sem root) + grupo plugdev..."
sudo tee /etc/udev/rules.d/51-android.rules >/dev/null <<'EOF'
# Test farm: acesso amplo a dispositivos USB (celulares) para o grupo plugdev.
SUBSYSTEM=="usb", ENV{DEVTYPE}=="usb_device", MODE="0666", GROUP="plugdev"
EOF
sudo udevadm control --reload-rules && sudo udevadm trigger || true
sudo usermod -aG plugdev "$USER" || true
adb kill-server 2>/dev/null || true
adb start-server 2>/dev/null || true

# 3. Arquitetura do runner ---------------------------------------------------
case "$(uname -m)" in
  aarch64|arm64) ARCH=arm64 ;;
  armv7l|armhf)  ARCH=arm ;;
  x86_64)        ARCH=x64 ;;
  *) die "Arquitetura não suportada: $(uname -m)" ;;
esac
log "Arquitetura detectada: $ARCH"

# 4. Baixa a última versão do GitHub Actions runner --------------------------
VER=$(curl -fsSL https://api.github.com/repos/actions/runner/releases/latest | jq -r .tag_name | sed 's/^v//')
[ -n "$VER" ] || die "Não consegui descobrir a versão do runner."
mkdir -p "$HOME/actions-runner" && cd "$HOME/actions-runner"
if [ ! -x ./config.sh ]; then
  TAR="actions-runner-linux-${ARCH}-${VER}.tar.gz"
  log "Baixando runner $VER ($ARCH)..."
  curl -fsSL -o "$TAR" "https://github.com/actions/runner/releases/download/v${VER}/${TAR}"
  tar xzf "$TAR" && rm -f "$TAR"
fi

# 5. Token de registro -------------------------------------------------------
if [ -z "${RUNNER_TOKEN:-}" ]; then
  echo
  echo "Gere um token novo (validade ~1h) em:"
  echo "  ${REPO_URL}/settings/actions/runners/new"
  echo "Copie APENAS o valor após --token."
  printf "Cole o RUNNER_TOKEN e Enter: "
  read -r RUNNER_TOKEN </dev/tty
fi
[ -n "$RUNNER_TOKEN" ] || die "Token vazio."

# 6. Registra e sobe como serviço (systemd) ----------------------------------
log "Registrando runner '$RUNNER_NAME' (labels: $RUNNER_LABELS)..."
./config.sh --url "$REPO_URL" --token "$RUNNER_TOKEN" \
  --labels "$RUNNER_LABELS" --name "$RUNNER_NAME" --unattended --replace
sudo ./svc.sh install "$USER"
sudo ./svc.sh start

log "PRONTO! ✅"
echo "  • Runner online: ${REPO_URL}/settings/actions/runners (deve aparecer '$RUNNER_NAME' Idle)"
echo "  • Conecte os celulares no Pi e rode: adb devices"
echo "  • Se 'adb devices' mostrar 'no permissions', deslogue/relogue (grupo plugdev) ou reinicie o Pi."
echo "  • Disparar teste: GitHub → Actions → deploy-devices → Run workflow."
