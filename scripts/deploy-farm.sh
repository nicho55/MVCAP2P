#!/usr/bin/env bash
# deploy-farm.sh — instala + inicia o APK em TODOS os devices adb conectados
# (USB ou WiFi), coleta modelo/Android/fps/memória e uma screenshot por aparelho.
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

  # Zera métricas e inicia pelo LAUNCHER (robusto p/ NativeActivity).
  # Empurra config file para testes automatizados (--gm --demo, screenshot, exit).
  ARGS_JSON='{"gm":true,"demo":true,"code":"TESTE","shot":"/data/local/tmp/tabletop_shot.png","shot_at":10.0,"exit_at":'$LAUNCH_SECONDS'.0}'
  echo "$ARGS_JSON" | adb -s "$S" shell "cat > /data/local/tmp/tabletop_args.json"
  adb -s "$S" shell dumpsys gfxinfo "$PKG" reset >/dev/null 2>&1 || true
  adb -s "$S" shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1

  echo "  ▶ rodando ${LAUNCH_SECONDS}s para coletar frames..."
  sleep "$LAUNCH_SECONDS"

  # O app crashou? (pid some se caiu — típico do Vulkan em GPU velha)
  if [ -z "$(adb -s "$S" shell pidof "$PKG" | tr -d '\r')" ]; then
    echo "  ⚠️  app não está mais rodando (possível crash de backend gráfico)." >&2
    FAIL=1
  fi

  PID="$(adb -s "$S" shell pidof "$PKG" | tr -d '\r')"
  REP="$OUT/${TAG}.txt"
  {
    echo "device=$S model=$M android=$R api=$SDK abi=$A"
    echo "--- gfxinfo ---"
    adb -s "$S" shell dumpsys gfxinfo "$PKG" | sed -n '/Total frames rendered/,/Number Slow/p'
    echo "--- meminfo ---"
    adb -s "$S" shell dumpsys meminfo "$PKG" | grep -E "TOTAL( |:)" | head -1
    if [ -n "$PID" ]; then
      echo "--- últimas linhas do log do app ---"
      adb -s "$S" logcat -d -t 80 --pid="$PID" 2>/dev/null || true
    else
      echo "--- CRASH LOG ---"
      adb -s "$S" logcat -d -t 200 -s AndroidRuntime:E DEBUG:* libc:F tabletop:* 2>/dev/null || true
    fi
  } > "$REP" 2>&1
  adb -s "$S" exec-out screencap -p > "$OUT/${TAG}.png" 2>/dev/null || true
  adb -s "$S" pull /data/local/tmp/tabletop_shot.png "$OUT/${TAG}_app.png" 2>/dev/null || true
  echo "  ✓ relatório: $REP"
done

echo ""
echo "Relatórios em: $OUT"
exit "$FAIL"
