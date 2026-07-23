//! # Crate: shared
//!
//! Tipos de domínio e protocolo **sem dependência de engine** (Bevy).
//! Reutilizável pelo cliente Bevy e pelo servidor SpacetimeDB (WASM).
//! Comunicação serializada com `bincode` via o enum `Msg`. O GM é autoritativo:
//! jogadores enviam `*Req`, GM valida e faz broadcast da versão final.
//! (Ver ADR-009.)

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

pub mod limits {
    // Temporário: relaxado de 600→1024 durante desenvolvimento (até P4).
    pub const MAX_RAM_MB: u32 = 1024;
    pub const MAX_TOKEN_BYTES: usize = 256 * 1024;
    pub const MAX_TOKEN_PX: u32 = 256;
    pub const MAX_MAP_BYTES: usize = 2 * 1024 * 1024;
    pub const MAX_MAP_PX: u32 = 2048;
    pub const MAX_SOUND_BYTES: usize = 128 * 1024;
    pub const MAX_TEXT_BYTES: usize = 8 * 1024;
    pub const THUMB_PX: u32 = 64;
    pub const THUMB_MAX_BYTES: usize = 8 * 1024;
    pub const MAX_STORAGE_MB: u32 = 200;
}

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

/// Índice de cor na paleta — **sempre válido** (0..PALETTE.len()). Estados
/// inválidos são inexprimíveis: o construtor reduz qualquer `u8` ao intervalo,
/// e a (de)serialização passa pelo construtor (`serde(try_from/into = u8)`),
/// então na rede continua sendo 1 byte, mas sem índice inválido possível.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(into = "u8", from = "u8")]
pub struct ColorIdx(u8);

impl ColorIdx {
    /// Reduz qualquer valor ao intervalo válido (wrap), garantindo o invariante.
    pub fn new(v: u8) -> Self {
        ColorIdx(v % PALETTE.len() as u8)
    }
    /// Índice cru, garantidamente em 0..PALETTE.len().
    pub fn get(self) -> u8 {
        self.0
    }
    /// Componentes RGB (0..1) da cor da paleta — indexação segura pelo
    /// invariante do tipo. O cliente converte para `bevy::Color`.
    pub fn rgb(self) -> [f32; 3] {
        let (r, g, b) = PALETTE[self.0 as usize];
        [r, g, b]
    }
}

impl From<ColorIdx> for u8 {
    fn from(c: ColorIdx) -> u8 {
        c.0
    }
}

impl From<u8> for ColorIdx {
    fn from(v: u8) -> Self {
        ColorIdx::new(v)
    }
}

/// Código curto de sala (ex.: `K7WM2`). **Sempre válido**: só caracteres do
/// `CODE_ALPHABET`, dentro do limite de tamanho. Nasce de `generate` (aleatório)
/// ou da entrada do usuário via `parse` (estrito) / `sanitized` (best-effort).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RoomCode(String);

impl RoomCode {
    /// Comprimento dos códigos gerados.
    pub const LEN: usize = 5;

    /// Gera um código aleatório válido.
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let s = (0..Self::LEN)
            .map(|_| CODE_ALPHABET[rng.gen_range(0..CODE_ALPHABET.len())] as char)
            .collect();
        RoomCode(s)
    }

    /// Valida estritamente: normaliza (trim + maiúsculas) e exige 3..=8
    /// caracteres, todos do `CODE_ALPHABET`. Retorna `None` se inválido.
    pub fn parse(s: &str) -> Option<Self> {
        let up = s.trim().to_uppercase();
        let ok = (3..=8).contains(&up.len()) && up.bytes().all(|b| CODE_ALPHABET.contains(&b));
        ok.then_some(RoomCode(up))
    }

    /// Melhor esforço: mantém só caracteres válidos (maiúsculas), limita o
    /// tamanho e, se sobrar vazio, gera um novo. Nunca falha.
    pub fn sanitized(s: &str) -> Self {
        let up: String = s
            .trim()
            .to_uppercase()
            .bytes()
            .filter(|b| CODE_ALPHABET.contains(b))
            .take(8)
            .map(char::from)
            .collect();
        if up.is_empty() {
            Self::generate()
        } else {
            RoomCode(up)
        }
    }

    /// Referência à string do código (para URL, Supabase, etc.).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RoomCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Remonta um blob a partir das partes recebidas por chunk. `parts[seq]` é o
/// pedaço da posição `seq` (ordem de chegada é irrelevante). Retorna `None`
/// enquanto qualquer parte ainda faltar.
pub fn reassemble_blob(parts: &[Option<Vec<u8>>]) -> Option<Vec<u8>> {
    if parts.iter().any(Option::is_none) {
        return None;
    }
    Some(parts.iter().flatten().flatten().copied().collect())
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
        Self {
            kind: GridKind::Square,
            cell: 64.0,
        }
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
    pub color: ColorIdx,
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
    BlobStart {
        id: BlobId,
        kind: BlobKind,
        len: u32,
        chunks: u32,
    },
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
    Terrain {
        cell: Cell,
        val: Option<TerrainCell>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_rgb_never_panics_and_wraps() {
        // Qualquer u8 vira um índice válido sem pânico...
        for i in 0u8..=255 {
            let _ = ColorIdx::new(i).rgb();
        }
        // ...e um índice além do tamanho volta ao início (wrap seguro).
        assert_eq!(
            ColorIdx::new(0).rgb(),
            ColorIdx::new(PALETTE.len() as u8).rgb()
        );
    }

    #[test]
    fn msg_bincode_roundtrip_is_stable() {
        // Serializar -> desserializar -> serializar deve dar os mesmos bytes.
        // (Msg não deriva PartialEq, então comparamos os bytes.)
        let msgs = [
            Msg::PlayerLeft(42),
            Msg::MoveTokenReq {
                id: 7,
                cell: (-3, 5),
            },
            Msg::TokenMoved {
                id: 7,
                cell: (10, -2),
            },
            Msg::RemoveToken { id: 99 },
            Msg::AssignToken {
                id: 1,
                new_owner: 1234,
            },
            Msg::DragPreview {
                id: 2,
                x: 1.5,
                y: -2.5,
            },
            Msg::Grid(GridCfg {
                kind: GridKind::HexFlat,
                cell: 48.0,
            }),
            Msg::BlobChunk {
                id: 5,
                seq: 3,
                data: vec![1, 2, 3, 4],
            },
        ];
        for m in msgs {
            let bytes = bincode::serialize(&m).expect("serialize");
            let back: Msg = bincode::deserialize(&bytes).expect("deserialize");
            let bytes2 = bincode::serialize(&back).expect("reserialize");
            assert_eq!(bytes, bytes2, "roundtrip instável para {m:?}");
        }
    }

    #[test]
    fn blob_reassembles_regardless_of_fill_order() {
        // Dados maiores que 2 chunks para exercitar o fatiamento real.
        let data: Vec<u8> = (0..(CHUNK * 2 + 123)).map(|i| (i % 251) as u8).collect();
        let chunks: Vec<Vec<u8>> = data.chunks(CHUNK).map(<[u8]>::to_vec).collect();
        let n = chunks.len();
        assert!(n >= 3, "esperado múltiplos chunks");

        let mut parts: Vec<Option<Vec<u8>>> = vec![None; n];
        // Enquanto faltar parte, remontar deve dar None.
        parts[n - 1] = Some(chunks[n - 1].clone());
        assert_eq!(reassemble_blob(&parts), None);

        // Preenche na ORDEM INVERSA; ao completar, remonta idêntico ao original.
        for seq in (0..n).rev() {
            parts[seq] = Some(chunks[seq].clone());
        }
        assert_eq!(reassemble_blob(&parts).as_deref(), Some(data.as_slice()));
    }

    #[test]
    fn color_idx_is_always_valid() {
        for v in 0u8..=255 {
            let c = ColorIdx::new(v);
            assert!((c.get() as usize) < PALETTE.len());
            let _ = c.rgb(); // nunca entra em pânico
        }
        assert_eq!(
            ColorIdx::new(0).rgb(),
            ColorIdx::new(PALETTE.len() as u8).rgb()
        );
    }

    #[test]
    fn color_idx_serde_is_one_byte_and_validates() {
        // Serializa como u8 puro (1 byte)...
        let bytes = bincode::serialize(&ColorIdx::new(3)).expect("serialize");
        assert_eq!(bytes.len(), 1);
        // ...e um byte "inválido" na rede (200) é reduzido ao intervalo ao ler.
        let wire = bincode::serialize(&200u8).expect("serialize u8");
        let c: ColorIdx = bincode::deserialize(&wire).expect("deserialize");
        assert!((c.get() as usize) < PALETTE.len());
    }

    #[test]
    fn room_code_generate_is_always_valid() {
        for _ in 0..100 {
            let c = RoomCode::generate();
            assert_eq!(c.as_str().len(), RoomCode::LEN);
            assert!(RoomCode::parse(c.as_str()).is_some());
        }
    }

    #[test]
    fn room_code_parse_validates_and_normalizes() {
        assert_eq!(RoomCode::parse("  k7wm2 ").unwrap().as_str(), "K7WM2"); // trim+upper
        assert!(RoomCode::parse("ab").is_none()); // curto demais
        assert!(RoomCode::parse("ABCDEFGHI").is_none()); // longo demais
        assert!(RoomCode::parse("K7W-2").is_none()); // char fora do alfabeto
        assert!(RoomCode::parse("K0O1I").is_none()); // 0/O/1/I não estão no alfabeto
    }

    #[test]
    fn room_code_sanitized_never_fails() {
        assert_eq!(RoomCode::sanitized("k7-w!m2").as_str(), "K7WM2"); // filtra inválidos
        assert!(RoomCode::parse(RoomCode::sanitized("!@#").as_str()).is_some());
        // garbage -> gera
    }
}
