#!/usr/bin/env bash
# deploy-farm.sh — instala + inicia o APK em TODOS os devices adb conectados
# (USB ou WiFi), captura screenshots multi-tela e coleta métricas.
# Uso: ./scripts/deploy-farm.sh   (env: APK, OUT, LAUNCH_SECONDS)
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
# shellcheck source=/dev/null
source "$SCRIPT_DIR/android-env.sh"

PKG="com.tabletop2p"
APK="${APK:-$PROJECT_DIR/app/android/app/build/outputs/apk/debug/app-debug.apk}"
OUT="${OUT:-$PROJECT_DIR/reports}"
LAUNCH_SECONDS="${LAUNCH_SECONDS:-20}"
mkdir -p "$OUT"

if [ ! -f "$APK" ]; then
  echo "❌ APK não encontrado: $APK" >&2
  exit 1
fi

# Todos os devices no estado "device" (ignora offline/unauthorized).
mapfile -t SERIALS < <(adb devices | awk 'NR>1 && $2=="device"{print $1}')
if [ "${#SERIALS[@]}" -eq 0 ]; then
  echo "❌ Nenhum device conectado (adb devices vazio)." >&2
  exit 1
fi

echo "📱 Devices: ${SERIALS[*]}"
FAIL=0

push_args() {
  echo "$2" | adb -s "$1" shell "cat > /data/local/tmp/tabletop_args.json"
}

launch_app() {
  adb -s "$1" shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1
}

stop_app() {
  adb -s "$1" shell am force-stop "$PKG" 2>/dev/null || true
  sleep 1
}

screencap() {
  adb -s "$1" exec-out screencap -p > "$2" 2>/dev/null || true
}

set_rotation() {
  adb -s "$1" shell settings put system accelerometer_rotation 0 2>/dev/null || true
  adb -s "$1" shell settings put system user_rotation "$2" 2>/dev/null || true
}

for S in "${SERIALS[@]}"; do
  M=$(adb -s "$S" shell getprop ro.product.model      | tr -d '\r')
  R=$(adb -s "$S" shell getprop ro.build.version.release | tr -d '\r')
  SDK=$(adb -s "$S" shell getprop ro.build.version.sdk | tr -d '\r')
  A=$(adb -s "$S" shell getprop ro.product.cpu.abi     | tr -d '\r')
  TAG="${M// /_}_${S}"
  echo "── $S | $M | Android $R (API $SDK) | $A"

  # minSdk do projeto = 24. Avisa se o aparelho for mais velho.
  if [ -n "$SDK" ] && [ "$SDK" -lt 24 ] 2>/dev/null; then
    echo "  ⚠️  API $SDK < minSdk 24 — APK NÃO instala aqui. Pular." >&2
    FAIL=1; continue
  fi

  # Instala; se assinatura incompatível, desinstala e reinstala.
  if ! adb -s "$S" install -r "$APK" 2>&1 | grep -q Success; then
    adb -s "$S" uninstall "$PKG" >/dev/null 2>&1 || true
    if ! adb -s "$S" install "$APK" 2>&1 | grep -q Success; then
      echo "  ✗ falha ao instalar" >&2
      FAIL=1; continue
    fi
  fi

  # Forçar landscape antes de começar
  set_rotation "$S" 1
  sleep 1

  # ── Rodada 1/4: Lobby ──────────────────────────────────────────
  echo "  📸 1/4 Lobby"
  stop_app "$S"
  push_args "$S" '{}'
  launch_app "$S"
  sleep 3
  screencap "$S" "$OUT/${TAG}_01-lobby.png"
  stop_app "$S"

  # ── Rodada 2/4: Jogo landscape ─────────────────────────────────
  echo "  📸 2/4 Jogo landscape"
  adb -s "$S" logcat -c 2>/dev/null || true
  EXIT_AT=$((LAUNCH_SECONDS + 30))
  push_args "$S" '{"gm":true,"demo":true,"code":"VISUAL","exit_at":'"$EXIT_AT"'.0}'
  adb -s "$S" shell dumpsys gfxinfo "$PKG" reset >/dev/null 2>&1 || true
  launch_app "$S"
  sleep 2
  LAUNCH_PID="$(adb -s "$S" shell pidof "$PKG" | tr -d '\r')"
  if [ -z "$LAUNCH_PID" ]; then
    echo "  ⚠️  app não iniciou (PID não encontrado)." >&2
    FAIL=1
  else
    echo "  ▶ PID=$LAUNCH_PID"
  fi
  sleep 4
  screencap "$S" "$OUT/${TAG}_02-game-landscape.png"

  # ── Rodada 3/4: Jogo portrait ──────────────────────────────────
  echo "  📸 3/4 Jogo portrait"
  set_rotation "$S" 0
  sleep 2
  screencap "$S" "$OUT/${TAG}_03-game-portrait.png"

  # ── Rodada 4/4: Jogo landscape restaurado ──────────────────────
  echo "  📸 4/4 Jogo landscape restaurado"
  set_rotation "$S" 1
  sleep 2
  screencap "$S" "$OUT/${TAG}_04-game-restored.png"

  # Esperar restante do LAUNCH_SECONDS para métricas (rodadas gastam ~14s)
  REMAINING=$((LAUNCH_SECONDS - 14))
  if [ "$REMAINING" -gt 0 ]; then
    echo "  ⏳ ${REMAINING}s restantes para coleta de métricas..."
    sleep "$REMAINING"
  fi

  # Verifica se app ainda está rodando
  PID="$(adb -s "$S" shell pidof "$PKG" | tr -d '\r')"
  if [ -z "$PID" ]; then
    echo "  ⚠️  app não está mais rodando (crash)." >&2
    FAIL=1
  fi

  # Coleta métricas e logs
  REP="$OUT/${TAG}.txt"
  {
    echo "device=$S model=$M android=$R api=$SDK abi=$A"
    echo "--- gfxinfo ---"
    adb -s "$S" shell dumpsys gfxinfo "$PKG" | sed -n '/Total frames rendered/,/Number Slow/p'
    echo "--- meminfo ---"
    adb -s "$S" shell dumpsys meminfo "$PKG" | grep -E "TOTAL( |:)" | head -1
    if [ -n "$PID" ]; then
      echo "--- log do app (rodando, PID=$PID) ---"
      adb -s "$S" logcat -d -t 80 --pid="$PID" 2>/dev/null || true
    elif [ -n "$LAUNCH_PID" ]; then
      echo "--- log do app (crashou, PID original=$LAUNCH_PID) ---"
      adb -s "$S" logcat -d --pid="$LAUNCH_PID" 2>/dev/null || true
      echo "--- crash logs (AndroidRuntime/DEBUG/libc) ---"
      adb -s "$S" logcat -d -s AndroidRuntime:E DEBUG:* libc:F 2>/dev/null || true
    else
      echo "--- app nunca iniciou ---"
    fi
    echo "--- logcat completo (últimas 300 linhas, sem filtro) ---"
    adb -s "$S" logcat -d -t 300 2>/dev/null | tail -300 || true
  } > "$REP" 2>&1

  stop_app "$S"

  # Restaurar auto-rotate
  adb -s "$S" shell settings put system accelerometer_rotation 1 2>/dev/null || true

  echo "  ✓ relatório: $REP"
  echo "  ✓ screenshots: 01-lobby, 02-landscape, 03-portrait, 04-restored"
done

echo ""
echo "Relatórios em: $OUT"
exit "$FAIL"
