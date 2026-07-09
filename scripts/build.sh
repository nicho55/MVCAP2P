#!/bin/bash
# build.sh — Build Linux desktop + Android APK
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
START=$(date +%s)

echo "═══════════════════════════════════════"
echo "  TABLETOP P2P — BUILD COMPLETO"
echo "═══════════════════════════════════════"

# ── 1. Desktop Linux ──
echo ""
echo "▸ [1/3] Build Linux (release)..."
cd "$PROJECT_DIR"
cargo build --release --package tabletop
echo "  ✓ Linux: target/release/tabletop"

# ── 2. Android APK ──
echo ""
echo "▸ [2/3] Build Android APK..."
source "$SCRIPT_DIR/android-env.sh"
cargo ndk -t arm64-v8a -P 24 -o app/android/app/src/main/jniLibs build --release --package tabletop 2>&1
cd app/android
./gradlew assembleDebug 2>&1
cd "$PROJECT_DIR"
APK="app/android/app/build/outputs/apk/debug/app-debug.apk"
echo "  ✓ APK: $APK"

# ── 3. Instalação via ADB (roda no HOST) ──
echo ""
echo "▸ [3/3] Finalizado"
echo "  ✓ APK gerado: $APK"
echo ""
echo "  Para instalar no celular, rode no HOST (fora do container):"
echo "    ./scripts/install-host.sh"

# ── Resumo ──
ELAPSED=$(( $(date +%s) - START ))
echo ""
echo "═══════════════════════════════════════"
echo "  BUILD CONCLUÍDO em ${ELAPSED}s"
echo "═══════════════════════════════════════"
echo "  Linux:  target/release/tabletop"
echo "  APK:    $APK"
echo ""
