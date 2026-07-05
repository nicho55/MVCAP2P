# Tabletop P2P — pacote de release

Binários Linux x86-64 (portáveis: compilados no baseline `x86-64` do rustc, só
SSE2 — sem `target-cpu=native`). Rodam em qualquer PC 64 bits. Todos os assets
gráficos e a fonte (DejaVu, com acentos) estão **embutidos** nos binários —
esta pasta é autossuficiente, não precisa de pasta `assets/` ao lado.

> Requisito mínimo: CPU x86-64 (qualquer uma de 64 bits) + GPU com **Vulkan**.
> As dependências que usam AVX2 fazem detecção em runtime e caem para SSE
> quando a CPU não tem AVX2, então CPUs antigas também funcionam.

```
tabletop        # o jogo (GM e jogadores)
signaling       # servidor de sinalização WebRTC (rode 1 instância só)
tabletop.sh     # launcher do jogo
signaling.sh    # launcher da sinalização
```

## Dependências de sistema (runtime)

Bibliotecas compartilhadas usadas pelo Bevy. No **Debian** e no **Linux Mint**
quase tudo já vem instalado; se faltar algo, instale:

```bash
# Linux Mint / Ubuntu
sudo apt install libasound2t64 libudev1 libxkbcommon0 libwayland-client0 \
                 libvulkan1 mesa-vulkan-drivers libx11-6 libxcb1 libxcursor1 \
                 libxi6 libxrandr2
```

> Precisa de **Vulkan** funcionando (GPU). Em Intel/AMD, `mesa-vulkan-drivers`
> resolve; em NVIDIA, o driver proprietário já traz o suporte Vulkan.
> Teste com `vulkaninfo | grep deviceName` (pacote `vulkan-tools`).

## Como jogar em 2 máquinas na mesma rede

Digamos que **esta máquina** tenha IP `192.168.1.10` e a outra (Mint) esteja na
mesma rede local.

**1) Numa máquina (qualquer uma), suba a sinalização:**
```bash
./signaling.sh
# -> sinalização WebRTC ouvindo em ws://0.0.0.0:3536
```
Descubra o IP dela: `ip addr | grep 'inet 192'` (ex.: `192.168.1.10`).

**2) O Mestre (nesta máquina) cria a sala:**
```bash
TABLETOP_SIGNALING=192.168.1.10:3536 ./tabletop.sh
# clique em "CRIAR SALA (MESTRE)" — anote o código exibido (ex.: K7WM2)
```

**3) O jogador (na Mint) entra:**
```bash
TABLETOP_SIGNALING=192.168.1.10:3536 ./tabletop.sh
# digite o código da sala e clique "ENTRAR"
```

Se a sinalização roda na própria máquina do Mestre, ele pode omitir a variável
(usa `127.0.0.1:3536` por padrão). Só os **jogadores remotos** precisam apontar
o `TABLETOP_SIGNALING` para o IP de quem hospeda a sinalização.

> Libere a porta **3536/tcp** no firewall da máquina que roda a sinalização:
> `sudo ufw allow 3536/tcp` (se usar ufw).

## Controles

- **Botão direito / WASD**: mover a câmera pelo mapa
- **Botão do meio / Q,E**: girar a câmera (orbitar)
- **Scroll**: zoom
- **Clique esquerdo**: selecionar / arrastar token (com snap ao grid)
- **Delete**: remover token selecionado
- **F12**: screenshot (salva na pasta atual)
- **Arrastar imagem PNG/JPEG/WebP** para a janela: cria token (ou troca o mapa,
  se o modo "MAPA" estiver ativo na toolbar — só o Mestre)

O Mestre tem a toolbar completa (pintar terreno, elevar/rebaixar, alternar grid
quadrado/hexagonal, ajustar célula, trocar mapa). O jogador move só os próprios
tokens; o Mestre move qualquer um.

## Linha de comando (opcional)

```
--gm                 cria sala direto (pula o lobby)
--join CODIGO        entra direto na sala
--nick Nome          apelido      --color 0..7   cor
--code ABCDE         força o código da sala (com --gm)
--demo               popula tokens de demonstração (Mestre)
--map arquivo.png    carrega um mapa na entrada (Mestre)
--signaling host:porta   (equivale à variável TABLETOP_SIGNALING)
```
