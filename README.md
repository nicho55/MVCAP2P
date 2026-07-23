# Tabletop P2P

VTT (Virtual Tabletop) tático peer-to-peer **3D low-poly** — "um Baldur's Gate 3,
mas é um VTT". Rust + [Bevy 0.16], rede P2P via WebRTC ([matchbox]), sem servidor
de jogo: o **Mestre (GM) é o host autoritativo**.

A cena é 3D: câmera orbital, terreno em prismas low-poly (deformar = altura real
da coluna, e os tokens sobem junto), tokens como peças de tabuleiro 3D (puck +
disco de arte + anel da cor do dono), árvores low-poly procedurais, luz
direcional com sombras. As artes 2D (texturas, faces dos tokens, mapa, ícones)
são SVGs gerados no repositório (`assets/svg/`), rasterizados em runtime com
`resvg` e aplicados como texturas nos meshes.

## Release para 2+ máquinas

`dist/` traz binários portáveis (baseline x86-64, assets e fonte embutidos) e
o `dist/LEIA-ME.md` com o passo a passo de jogar em rede local. Gerar:

```bash
cargo build --release        # binários em target/release/{tabletop,signaling}
# empacotados em dist/ + tarball
```

## Rodando

```bash
# 1. servidor de sinalização WebRTC (só troca handshakes; nenhum dado de jogo passa por ele)
cargo run -p signaling            # ouve em ws://0.0.0.0:3536

# 2. o mestre
cargo run -p tabletop             # clique em "CRIAR SALA (MESTRE)"

# 3. cada jogador
cargo run -p tabletop             # digite o código da sala e "ENTRAR"
```

Sinalização em outra máquina: `TABLETOP_SIGNALING=192.168.1.10:3536 cargo run -p tabletop`
(ou `--signaling host:porta`).

## Funcionalidades

### Salas & rede
- Sala com código curto (ex.: `K7WM2`); jogadores entram pelo código.
- WebRTC DataChannel (confiável/ordenado) entre todos os peers; GM é autoridade:
  jogadores enviam *requests* (mover/criar/remover token), o GM valida e faz broadcast.
- Reconexão automática: caiu a sinalização → retry com backoff; GM voltou → re-Hello + resync completo (Welcome).
- Identificação visual: apelido + cor escolhidos no lobby; anel colorido em cada token com a cor do dono.

### Mapa & grid
- Importação de mapa por **arrastar e soltar** arquivo (PNG/JPEG/WebP) na janela
  (com o modo de drop "MAPA" ativo na toolbar — só o GM). A imagem é fatiada em chunks
  de 14 KB e replicada aos peers.
- Mapa padrão (pergaminho com ruínas/rio/trilha) gerado de SVG.
- Grid **quadrado** e **hexagonal flat-top** (coordenadas axiais), alternável pelo GM,
  tamanho de célula ajustável (+/−). Snap-to-grid em ambos.
- Câmera: pan com botão direito/meio, zoom no scroll ancorado no cursor.

### Tokens
- Criar token de imagem importada (drop com modo "TOKEN") — jogadores também podem.
- 4 tokens embutidos (guerreiro, mago, ladino, dragão) — GM: `--demo`.
- Arrastar com snap; posição sincronizada em tempo real (preview de drag a 20 Hz + snap final).
- Seleção com anel dourado; `Delete` remove (dono ou GM).
- GM move qualquer token; jogador só os seus (validado no GM, não só na UI).

### Terreno (GM)
- Pintura de texturas por célula: grama, pedra, água, areia (toolbar) + borracha.
- **Deformação**: elevar/rebaixar célula (−4..+4), renderizado por sombreamento.
- Tudo sincronizado e incluído no snapshot de entrada de novos jogadores.

## CLI (automação/testes)

```
--gm                 cria sala direto (pula lobby)      --code ABCDE   força o código
--join ABCDE         entra direto na sala               --nick Nome    apelido
--color 0..7         cor                                --demo         tokens de demonstração (GM)
--map caminho.png    carrega mapa na entrada (GM)       --signaling host:porta
--shot arq.png       screenshot automático              --shot-at 6    segundos até o shot
--exit-at 12         encerra sozinho                    F12            screenshot manual
```

Teste local completo em 3 terminais:

```bash
cargo run -p signaling
cargo run -p tabletop -- --gm --code TESTE --nick Mestre --demo
cargo run -p tabletop -- --join TESTE --nick Ana --color 1
```

## Arquitetura

```
signaling/            matchbox_signaling (full-mesh), só handshake WebRTC
app/src/
  protocol.rs         enum Msg (bincode/serde), chunking de blobs, paleta
  net.rs              socket matchbox, poll/reconexão, roster, blobs (imagens)
  svg_assets.rs       rasterização dos SVGs (resvg) -> texturas Bevy em PreStartup
  lobby.rs            UI de entrada (campos de texto próprios, swatches de cor)
  game/
    camera.rs         pan/zoom (Transform.scale, zoom ancorado no cursor)
    grid.rs           matemática quadrado/hex (axial + cube-round), gizmos, reflow
    map.rs            drop de arquivos, sprite do mapa, sync
    tokens.rs         spawn/drag/seleção/remoção, anéis de dono/seleção
    terrain.rs        pintura + elevação por célula, render de overlays
    hud.rs            toolbar por papel (GM/jogador), roster, status, dicas
    sync.rs           handlers de rede: Hello→Welcome, autoridade do GM
```

Decisões notáveis:
- **Bevy 0.18 + bevy_matchbox 0.14** fixados (upgrade realizado conforme ADR-011).
- Estado inteiro cabe num `Welcome` (players, grid, terreno, tokens, blob do mapa),
  então *join tardio* e *resync pós-reconexão* são o mesmo código.
- Propriedade de token por UUID de jogador (estável na sessão), não por PeerId
  (que muda a cada reconexão).
- **Identidade P2P local** (sem servidor externo): token 256-bit persistido em
  arquivo, username recall entre sessões. Ver ADR-011.
