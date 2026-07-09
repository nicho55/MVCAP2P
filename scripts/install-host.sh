#!/bin/bash
# install-host.sh — Roda no HOST (fora do container) pra instalar o APK via ADB
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
APK="$PROJECT_DIR/app/android/app/build/outputs/apk/debug/app-debug.apk"

echo ""
echo "═══════════════════════════════════════"
echo "  INSTALAR APK NO CELULAR"
echo "═══════════════════════════════════════"
echo ""

if ! command -v adb &>/dev/null; then
    echo "[1/4] adb nao encontrado."
    echo "   Instale o platform-tools do Android SDK (Sdk/platform-tools/)."
    exit 1
fi
echo "[1/4] adb encontrado: $(which adb)"

echo -n "[2/4] Procurando celular via USB... "
DEVICE_LINE=$(adb devices | grep -v "List of devices attached" | grep -v "^$" | head -1)
if [ -z "$DEVICE_LINE" ]; then
    echo "Nao encontrado"
    echo "   Nenhum celular conectado. Verifique:"
    echo "   - Cabo USB conectado"
    echo "   - Depuracao USB ativada no celular"
    echo "   - Confiar no computador (pop-up no celular)"
    exit 1
fi
DEVICE_SERIAL=$(echo "$DEVICE_LINE" | awk '{print $1}')
echo "OK"
echo "   Serial: $DEVICE_SERIAL"
DEVICE_MODEL=$(adb -s "$DEVICE_SERIAL" shell getprop ro.product.model 2>/dev/null | tr -d '\r' || echo "desconhecido")
echo "   Modelo: $DEVICE_MODEL"

echo -n "[3/4] Verificando APK... "
if [ ! -f "$APK" ]; then
    echo "Nao encontrado"
    echo "   APK nao encontrado em:"
    echo "   $APK"
    echo "   Rode ./scripts/build.sh no container primeiro."
    exit 1
fi
APK_SIZE=$(du -h "$APK" | cut -f1)
echo "OK ($APK_SIZE)"

echo "[4/4] Instalando APK no celular..."
echo "   Arquivo: $(basename "$APK")"
echo ""

START=$(date +%s)
OUTPUT=$(adb -s "$DEVICE_SERIAL" install -r "$APK" 2>&1) || true
ELAPSED=$(( $(date +%s) - START ))

echo ""
if echo "$OUTPUT" | grep -q "INSTALL_FAILED_UPDATE_INCOMPATIBLE"; then
    echo "  Assinatura incompativel -- desinstalando versao anterior..."
    adb -s "$DEVICE_SERIAL" uninstall com.tabletop2p 2>&1
    echo "  Tentando instalar novamente..."
    START=$(date +%s)
    OUTPUT=$(adb -s "$DEVICE_SERIAL" install -r "$APK" 2>&1) || true
    ELAPSED=$(( $(date +%s) - START ))
    echo ""
fi

if echo "$OUTPUT" | grep -q "Success"; then
    echo "Instalado com sucesso em ${ELAPSED}s!"
else
    echo "Falha na instalacao apos ${ELAPSED}s."
    echo "   $OUTPUT"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════"
echo "  PRONTO — app instalado no celular"
echo "═══════════════════════════════════════"
