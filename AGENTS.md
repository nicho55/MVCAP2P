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

## Padrões de Código (Type-Driven Design)

> Complementa as regras de estilo já vigentes (docstrings, sem `unwrap`, Rust
> idiomático). Justificativa e origem no **ADR-006**.

1. **Parse, don't validate — newtypes, não `type` aliases crus.**
   Prefira `struct RoomCode(String)` a `type PlayerUuid = u64`. A validação mora
   no construtor (`try_new`/`parse`); o resto do código confia no tipo.
   Alvos: código de sala, índice de cor (`0..8` da `PALETTE`), UUID de jogador,
   IDs de token/blob.

2. **Estados inválidos inexprimíveis — enums algébricos.**
   Modele variações como enum que carrega só os campos válidos do caso (padrão já
   usado em `Msg` e `GridKind`). Evite structs cheias de `Option` "conforme o
   caso". Papel (GM/Jogador) e modo de drop (MAPA/TOKEN) são tipos, não `bool` +
   checagem espalhada.

3. **Typestate para ciclo de vida.**
   O ciclo `Boot → Lobby → InGame` já é `AppState` (Bevy States) — trate-o como o
   padrão oficial. Operações só devem existir no estado que as permite (ex.:
   broadcast apenas com socket conectado).

4. **Erros com `thiserror` + `?`.**
   Introduza enums de erro (ex.: `NetError`, `ProtocolError`) com `thiserror` e
   propague com `?`. Zero `unwrap()`/`expect()` no caminho de rede.

5. **`#[non_exhaustive]` no que evolui.**
   `Msg` é protocolo versionado: `#[non_exhaustive]` nos enums públicos de
   protocolo e de erro, para não quebrar peers ao adicionar variantes. Structs
   públicas complexas ganham construtor/builder em vez de construção crua.

6. **Sealed traits para extensão controlada.**
   Traits de comportamento interno (serialização de mensagem, cálculo de layout)
   são *sealed*: só nossos tipos as implementam.

### Portões de qualidade (força o cumprimento)
- Configs na raiz: `rustfmt.toml`, `clippy.toml`, `deny.toml`.
- Antes de concluir uma mudança:
  `cargo fmt` · `cargo clippy --all-targets --all-features -- -D warnings` · `cargo test`.
- CI em `.github/workflows/ci.yml` roda fmt + clippy + test + doc a cada push/PR.
