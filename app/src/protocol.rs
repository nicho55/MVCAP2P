//! # Módulo: protocol
//!
//! Define os tipos de dados e mensagens trocadas entre peers via WebRTC.
//! Toda comunicação serializada com `bincode` usa os tipos e enum `Msg`
//! declarados aqui. O GM é autoritativo: jogadores enviam `*Req`, GM
//! valida e broadcast a versão final.

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

/// Identificador único de jogador (gerado aleatoriamente a cada sessão).
pub type PlayerUuid = u64;

/// Identificador único de token de jogo.
pub type TokenId = u64;

/// Identificador único de blob (imagem transferida em chunks).
pub type BlobId = u64;

/// Coordenada de célula no grid (coluna, linha).
pub type Cell = (i32, i32);

/// Alfabeto para códigos de sala (exclui I/1/O/0 para evitar confusão).
pub const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Tamanho de cada chunk de blob em bytes (14 KB).
pub const CHUNK: usize = 14 * 1024;

/// Valor sentinela para textura vazia/nenhuma.
pub const TEX_NONE: u8 = 255;

/// Paleta de 8 cores para os jogadores (RGB normalizado 0-1).
pub const PALETTE: [(f32, f32, f32); 8] = [
    (0.898, 0.282, 0.302), // vermelho
    (0.243, 0.388, 0.867), // azul
    (0.275, 0.655, 0.345), // verde
    (0.961, 0.851, 0.039), // amarelo
    (0.557, 0.306, 0.776), // roxo
    (0.969, 0.420, 0.082), // laranja
    (0.020, 0.635, 0.761), // ciano
    (0.839, 0.251, 0.624), // rosa
];

/// Retorna uma cor da paleta pelo índice (com wrap seguro).
pub fn palette_color(i: u8) -> Color {
    let (r, g, b) = PALETTE[i as usize % PALETTE.len()];
    Color::srgb(r, g, b)
}

/// Tipo de grid do tabuleiro.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridKind {
    #[default]
    /// Grid quadrado.
    Square,
    /// Grid hexagonal (flat-top).
    HexFlat,
}

/// Configuração do grid: tipo e tamanho da célula em pixels.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct GridCfg {
    pub kind: GridKind,
    /// Largura/altura de cada célula em unidades do mundo.
    pub cell: f32,
}

impl Default for GridCfg {
    fn default() -> Self {
        Self { kind: GridKind::Square, cell: 64.0 }
    }
}

/// Célula de terreno: textura e elevação.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct TerrainCell {
    /// Índice da textura (TEX_NONE = vazio).
    pub tex: u8,
    /// Elevação em unidades (positivo = para cima).
    pub elev: i8,
}

/// Arte de um token: textura embutida ou blob transferido.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenArt {
    /// Textura pré-definida (índice do conjunto embutido).
    BuiltIn(u8),
    /// Imagem personalizada transferida via blob.
    Blob(BlobId),
}

/// Metadados de um token: identificadores, dono, arte e posição.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenMeta {
    pub id: TokenId,
    /// UUID do jogador dono do token.
    pub owner: PlayerUuid,
    pub art: TokenArt,
    /// Coordenada da célula onde o token está.
    pub cell: Cell,
}

/// Metadados de um jogador: identificadores, apelido e papel na mesa.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerMeta {
    pub uuid: PlayerUuid,
    pub nick: String,
    /// Índice da cor na paleta.
    pub color: u8,
    /// Se é o Mestre (autoridade do jogo).
    pub is_gm: bool,
}

/// Tipo de blob transferido entre peers.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlobKind {
    /// Imagem (mapa ou arte de token).
    Image,
}

/// Mensagens trocadas entre peers via WebRTC (serializadas com bincode).
///
/// O GM é autoritativo: jogadores enviam pedidos (`*Req`), o GM valida
/// e transmite a versão final (`TokenMoved`, `SpawnToken`, etc.).
/// Blobs (imagens) são enviados em chunks antes do `Welcome` para
/// garantir que estejam disponíveis ao processar o estado inicial.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Msg {
    /// Jogador se apresenta ao GM (enviado ao conectar).
    Hello(PlayerMeta),
    /// GM envia estado completo do jogo para o novo jogador.
    Welcome {
        players: Vec<PlayerMeta>,
        grid: GridCfg,
        terrain: Vec<(Cell, TerrainCell)>,
        tokens: Vec<TokenMeta>,
        map_blob: Option<BlobId>,
    },
    /// GM notifica todos que um novo jogador entrou.
    PlayerJoined(PlayerMeta),
    /// GM notifica que um jogador saiu (pelo UUID).
    PlayerLeft(PlayerUuid),
    /// GM altera configuração do grid.
    Grid(GridCfg),
    /// Início da transferência de um blob (cabeçalho com metadados).
    BlobStart { id: BlobId, kind: BlobKind, len: u32, chunks: u32 },
    /// Um chunk de dados do blob (sequência numerada).
    BlobChunk { id: BlobId, seq: u32, data: Vec<u8> },
    /// GM define/remove a imagem do mapa por blob ID.
    SetMap { blob: Option<BlobId> },
    /// Jogador pede ao GM para spawnar um token.
    SpawnTokenReq(TokenMeta),
    /// GM autoriza e broadcast do token spawnado.
    SpawnToken(TokenMeta),
    /// Jogador pede ao GM para mover um token.
    MoveTokenReq { id: TokenId, cell: Cell },
    /// GM autoriza e broadcast da nova posição.
    TokenMoved { id: TokenId, cell: Cell },
    /// Jogador pede ao GM para remover um token.
    RemoveTokenReq { id: TokenId },
    /// GM autoriza e broadcast da remoção.
    RemoveToken { id: TokenId },
    /// GM transfere a posse de um token para outro jogador.
    AssignToken { id: TokenId, new_owner: PlayerUuid },
    /// Preview de arrasto (broadcast throttled ~20Hz para todos).
    DragPreview { id: TokenId, x: f32, y: f32 },
    /// GM modifica uma célula de terreno (pintar/esculpir).
    Terrain { cell: Cell, val: Option<TerrainCell> },
}
