#!/bin/bash
# Driver autônomo para build + deploy Android.
# Tenta build em loop com backoff, reporta erros via checkpoint.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
CHECKPOINT="$PROJECT_DIR/scripts/android-port-status.json"
LOG_FILE="$PROJECT_DIR/scripts/android-build.log"
MAX_RETRIES=20
RETRY_DELAY=30  # segundos entre tentativas

# Carrega env Android
source "$SCRIPT_DIR/android-env.sh"

log() {
  local msg="[$(date '+%H:%M:%S')] $1"
  echo "$msg" | tee -a "$LOG_FILE"
}

update_checkpoint() {
  local step_id="$1"
  local status="$2"
  local log_msg="$3"
  tmp=$(mktemp)
  python3 -c "
import json
with open('$CHECKPOINT') as f:
  c = json.load(f)
for s in c['steps']:
  if s['id'] == $step_id:
    s['status'] = '$status'
    s['log'] = '$log_msg'
if '$status' == 'done':
  c['current_step'] = $step_id + 1
with open('$CHECKPOINT', 'w') as f:
  json.dump(c, f, indent=2)
" 2>/dev/null || true
  rm -f "$tmp"
}

# Detecta device
detect_device() {
  local serial
  serial=$(adb devices 2>/dev/null | grep -v "List" | grep -v "^$" | head -1 | awk '{print $1}')
  if [ -n "$serial" ]; then
    log "📱 Device detectado: $serial"
    tmp=$(mktemp)
    python3 -c "
import json
with open('$CHECKPOINT') as f:
  c = json.load(f)
c['device_serial'] = '$serial'
with open('$CHECKPOINT', 'w') as f:
  json.dump(c, f, indent=2)
" 2>/dev/null || true
    rm -f "$tmp"
    echo "$serial"
  else
    log "⚠️  Nenhum device encontrado via adb"
    echo ""
  fi
}

build_apk() {
  log "🔨 Build Rust para Android (arm64)..."
  cd "$PROJECT_DIR"
  cargo ndk -t arm64-v8a -o app/android/app/src/main/jniLibs build --release 2>> "$LOG_FILE"
}

build_gradle() {
  log "📦 Build APK com Gradle..."
  cd "$PROJECT_DIR/app/android"
  ./gradlew assembleDebug 2>> "$LOG_FILE"
}

deploy_apk() {
  local serial="$1"
  local apk_path="$PROJECT_DIR/app/android/app/build/outputs/apk/debug/app-debug.apk"
  if [ ! -f "$apk_path" ]; then
    log "❌ APK não encontrado em $apk_path"
    return 1
  fi
  log "📲 Instalando APK no device..."
  if [ -n "$serial" ]; then
    adb -s "$serial" install -r "$apk_path" 2>&1 | tee -a "$LOG_FILE"
  else
    adb install -r "$apk_path" 2>&1 | tee -a "$LOG_FILE"
  fi
  log "✅ App instalado! Iniciando..."
  # A activity é android.app.NativeActivity (hasCode=false). Iniciar pelo
  # LAUNCHER é robusto e independe do nome interno da activity.
  if [ -n "$serial" ]; then
    adb -s "$serial" shell monkey -p "com.tabletop2p" -c android.intent.category.LAUNCHER 1 2>&1 | tee -a "$LOG_FILE"
  else
    adb shell monkey -p "com.tabletop2p" -c android.intent.category.LAUNCHER 1 2>&1 | tee -a "$LOG_FILE"
  fi
  log "🎉 App rodando no celular!"
}

# ===== Main Loop =====
log "🚀 Iniciando Android Driver"
log "   Checkpoint: $CHECKPOINT"
log "   Log: $LOG_FILE"

device=$(detect_device)

attempt=0
while [ $attempt -lt $MAX_RETRIES ]; do
  attempt=$((attempt + 1))
  log ""
  log "═══════════════════════════════════"
  log "  Tentativa $attempt de $MAX_RETRIES"
  log "═══════════════════════════════════"

  if build_apk; then
    log "✅ Build Rust OK!"
    break
  else
    log "❌ Build Rust falhou (tentativa $attempt)"
    log "   Últimos erros no log: $LOG_FILE"
    update_checkpoint 8 "failed" "Build Rust falhou na tentativa $attempt"
    if [ $attempt -lt $MAX_RETRIES ]; then
      log "   Aguardando ${RETRY_DELAY}s antes de tentar de novo..."
      sleep $RETRY_DELAY
    fi
  fi
done

if [ $attempt -ge $MAX_RETRIES ]; then
  log "💀 Esgotadas $MAX_RETRIES tentativas de build."
  update_checkpoint 8 "failed" "Esgotadas $MAX_RETRIES tentativas"
  exit 1
fi

# Gradle build
if build_gradle; then
  log "✅ Gradle OK!"
else
  log "❌ Gradle falhou"
  exit 1
fi

# Deploy
if deploy_apk "$device"; then
  update_checkpoint 8 "done" "Build + deploy bem-sucedido"
  log "✅ PROCESSO COMPLETO — app rodando no celular!"
  exit 0
else
  log "❌ Deploy falhou"
  exit 1
fi
