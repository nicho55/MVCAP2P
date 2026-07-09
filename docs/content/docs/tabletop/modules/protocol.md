# `protocol`

**Path**: `src/protocol.rs`

## Descrição

 # Módulo: protocol

 Define os tipos de dados e mensagens trocadas entre peers via WebRTC.

 Toda comunicação serializada com `bincode` usa os tipos e enum `Msg`

 declarados aqui. O GM é autoritativo: jogadores enviam `*Req`, GM

 valida e broadcast a versão final.

## Structs

### `GridCfg`

 Configuração do grid: tipo e tamanho da célula em pixels.

**Derives**: Serialize, Deserialize, Clone, Copy, Debug, PartialEq

| Campo | Tipo |
|-------|------|
| `kind` | `GridKind` |
| `cell` | `f32` |

### `TerrainCell`

 Célula de terreno: textura e elevação.

**Derives**: Serialize, Deserialize, Clone, Copy, Debug, PartialEq

| Campo | Tipo |
|-------|------|
| `tex` | `u8` |
| `elev` | `i8` |

### `TokenMeta`

 Metadados de um token: identificadores, dono, arte e posição.

**Derives**: Serialize, Deserialize, Clone, Debug

| Campo | Tipo |
|-------|------|
| `id` | `TokenId` |
| `owner` | `PlayerUuid` |
| `art` | `TokenArt` |
| `cell` | `Cell` |

### `PlayerMeta`

 Metadados de um jogador: identificadores, apelido e papel na mesa.

**Derives**: Serialize, Deserialize, Clone, Debug

| Campo | Tipo |
|-------|------|
| `uuid` | `PlayerUuid` |
| `nick` | `String` |
| `color` | `u8` |
| `is_gm` | `bool` |

## Enums

### `GridKind`

 Tipo de grid do tabuleiro.

**Derives**: Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default

| Variante | Campos |
|----------|--------|
| `Square` | `—` |
| `HexFlat` | `—` |

### `TokenArt`

 Arte de um token: textura embutida ou blob transferido.

**Derives**: Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash

| Variante | Campos |
|----------|--------|
| `BuiltIn` | `u8` |
| `Blob` | `BlobId` |

### `BlobKind`

 Tipo de blob transferido entre peers.

**Derives**: Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash

| Variante | Campos |
|----------|--------|
| `Image` | `—` |

### `Msg`

 Mensagens trocadas entre peers via WebRTC (serializadas com bincode).



 O GM é autoritativo: jogadores enviam pedidos (`*Req`), o GM valida

 e transmite a versão final (`TokenMoved`, `SpawnToken`, etc.).

 Blobs (imagens) são enviados em chunks antes do `Welcome` para

 garantir que estejam disponíveis ao processar o estado inicial.

**Derives**: Serialize, Deserialize, Clone, Debug

| Variante | Campos |
|----------|--------|
| `Hello` | `PlayerMeta` |
| `Welcome` | `players: Vec < PlayerMeta >, grid: GridCfg, terrain: Vec < (Cell , TerrainCell) >, tokens: Vec < TokenMeta >, map_blob: Option < BlobId >` |
| `PlayerJoined` | `PlayerMeta` |
| `PlayerLeft` | `PlayerUuid` |
| `Grid` | `GridCfg` |
| `BlobStart` | `id: BlobId, kind: BlobKind, len: u32, chunks: u32` |
| `BlobChunk` | `id: BlobId, seq: u32, data: Vec < u8 >` |
| `SetMap` | `blob: Option < BlobId >` |
| `SpawnTokenReq` | `TokenMeta` |
| `SpawnToken` | `TokenMeta` |
| `MoveTokenReq` | `id: TokenId, cell: Cell` |
| `TokenMoved` | `id: TokenId, cell: Cell` |
| `RemoveTokenReq` | `id: TokenId` |
| `RemoveToken` | `id: TokenId` |
| `AssignToken` | `id: TokenId, new_owner: PlayerUuid` |
| `DragPreview` | `id: TokenId, x: f32, y: f32` |
| `Terrain` | `cell: Cell, val: Option < TerrainCell >` |

## Funções

### `palette_color`

```rust
fn palette_color(i : u8) -> Color
```

 Retorna uma cor da paleta pelo índice (com wrap seguro).

## Implementações

### `impl Default for GridCfg`

- `default`

## Constantes

| Nome | Tipo | Valor |
|------|------|-------|
| `CODE_ALPHABET` | `& [u8]` | `b"ABCDEFGHJKMNPQRSTUVWXYZ23456789"` |
| `CHUNK` | `usize` | `14 * 1024` |
| `TEX_NONE` | `u8` | `255` |
| `PALETTE` | `[(f32 , f32 , f32) ; 8]` | `[(0.898 , 0.282 , 0.302) , (0.243 , 0.388 , 0.867) , (0.275 , 0.655 , 0.345) , (0.961 , 0.851 , 0.039) , (0.557 , 0.306 , 0.776) , (0.969 , 0.420 , 0.082) , (0.020 , 0.635 , 0.761) , (0.839 , 0.251 , 0.624) ,]` |

## Type Aliases

| Nome | Tipo |
|------|------|
| `PlayerUuid` | `u64` |
| `TokenId` | `u64` |
| `BlobId` | `u64` |
| `Cell` | `(i32 , i32)` |

