use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

pub type PlayerUuid = u64;
pub type TokenId = u64;
pub type BlobId = u64;
pub type Cell = (i32, i32);

pub const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
pub const CHUNK: usize = 14 * 1024;
pub const TEX_NONE: u8 = 255;

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

pub fn palette_color(i: u8) -> Color {
    let (r, g, b) = PALETTE[i as usize % PALETTE.len()];
    Color::srgb(r, g, b)
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridKind {
    #[default]
    Square,
    HexFlat,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct GridCfg {
    pub kind: GridKind,
    pub cell: f32,
}

impl Default for GridCfg {
    fn default() -> Self {
        Self { kind: GridKind::Square, cell: 64.0 }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct TerrainCell {
    pub tex: u8,
    pub elev: i8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenArt {
    BuiltIn(u8),
    Blob(BlobId),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenMeta {
    pub id: TokenId,
    pub owner: PlayerUuid,
    pub art: TokenArt,
    pub cell: Cell,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerMeta {
    pub uuid: PlayerUuid,
    pub nick: String,
    pub color: u8,
    pub is_gm: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlobKind {
    Image,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Msg {
    Hello(PlayerMeta),
    Welcome {
        players: Vec<PlayerMeta>,
        grid: GridCfg,
        terrain: Vec<(Cell, TerrainCell)>,
        tokens: Vec<TokenMeta>,
        map_blob: Option<BlobId>,
    },
    PlayerJoined(PlayerMeta),
    PlayerLeft(PlayerUuid),
    Grid(GridCfg),
    BlobStart { id: BlobId, kind: BlobKind, len: u32, chunks: u32 },
    BlobChunk { id: BlobId, seq: u32, data: Vec<u8> },
    SetMap { blob: Option<BlobId> },
    SpawnTokenReq(TokenMeta),
    SpawnToken(TokenMeta),
    MoveTokenReq { id: TokenId, cell: Cell },
    TokenMoved { id: TokenId, cell: Cell },
    RemoveTokenReq { id: TokenId },
    RemoveToken { id: TokenId },
    DragPreview { id: TokenId, x: f32, y: f32 },
    Terrain { cell: Cell, val: Option<TerrainCell> },
}
