# Projeto: Tabletop P2P

VTT tático 3D low-poly peer-to-peer em Rust/Bevy.

## Estrutura

```
app/src/          — Crate principal (jogo)
  game/           — Lógica do jogo (grid, tokens, mapa, sync, etc.)
  net.rs          — Rede (WebRTC via Matchbox)
  protocol.rs     — Mensagens serializadas (Msg enum)
  lobby.rs        — Tela de lobby
  room_discovery  — Lista de salas via Supabase
signaling/        — Servidor de sinalização WebSocket
docgen/           — Gerador de documentação a partir do código
docs/             — Site de documentação (Next.js + Fumadocs)
scripts/          — Scripts de build e deploy
```

## Comandos Essenciais

### Build completo (Linux + Android + celular)
```bash
./scripts/build.sh
```

### Build separados
```bash
cargo build --release --package tabletop                            # Linux
cargo ndk -t arm64-v8a -o app/android/app/src/main/jniLibs build --release  # lib Android
cd app/android && ./gradlew assembleDebug                           # APK
```

### Servidor de sinalização
```bash
cargo run -p signaling
```

### Documentação local
```bash
cd docs && npm install && npm run dev     # http://localhost:3000
```

### Testar no celular (USB)
```bash
adb install -r app/android/app/build/outputs/apk/debug/app-debug.apk
```

### Rede (para jogar)
1. Mestre roda `cargo run -p signaling` em um terminal
2. Mestre executa `cargo run -- --gm`
3. Jogador executa `cargo run` com IP do servidor de sinalização
   ```bash
   TABLETOP_SIGNALING=192.168.x.x:3536 cargo run
   ```

## Otimizações de Build

- **sccache** — cache de compilação (evita recompilar dependências)
- **mold** — linker rápido (configurado via clang + -fuse-ld=mold)
- **Perfil release:** `lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"`
- **Cargo.ndk** — cross-compilação para Android ARM64
- **Dockerfile** — mold, clang, sccache, cargo-ndk já instalados

O primeiro build leva 15-30 min (Bevy + dependências). Builds subsequentes
são muito mais rápidos graças ao sccache.

## Convenções
- `protocol.rs` — novas mensagens seguem o padrão `Req → GM valida → broadcast`
- Tokens: jogador pede (`SpawnTokenReq`), GM autoriza e broadcast (`SpawnToken`)
- Imagens: enviadas em chunks de 14KB (`BlobStart` + `BlobChunk`)
