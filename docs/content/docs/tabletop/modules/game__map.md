# `map`

**Path**: `src/game/map.rs`

## Resources (Bevy)

### `MapState`

| Campo | Tipo |
|-------|------|
| `want` | `Option < BlobId >` |
| `applied` | `Option < Option < BlobId > >` |
| `size` | `Vec2` |

## Components (Bevy)

### `MapGround`

## Enums

### `DropMode`

**Derives**: Resource, Default, PartialEq, Clone, Copy

| Variante | Campos |
|----------|--------|
| `Token` | `—` |
| `Map` | `—` |

## Funções

### `import_map_bytes`

```rust
fn import_map_bytes(bytes : Vec < u8 >, blobs : & mut Blobs, images : & mut Assets < Image >, net : & mut Net, map_state : & mut MapState) -> ()
```

## Systems (Bevy)

### `sync_map`

 Aplica o mapa desejado: plano texturizado no chão (XZ), centrado na origem.

 No mapa padrão, decora com árvores low-poly (filhas do plano — somem juntas).

**Parâmetros**: `mut commands : Commands`, `mut map_state : ResMut < MapState >`, `blobs : Res < Blobs >`, `assets : Res < GameAssets >`, `images : Res < Assets < Image > >`, `q_map : Query < Entity , With < MapGround > >`, `mut meshes : ResMut < Assets < Mesh > >`, `mut ctx : Ctx3d`

### `file_drop`

**Parâmetros**: `mut evr : EventReader < FileDragAndDrop >`, `drop_mode : Res < DropMode >`, `session : Res < Session >`, `mut net : ResMut < Net >`, `mut blobs : ResMut < Blobs >`, `mut images : ResMut < Assets < Image > >`, `mut map_state : ResMut < MapState >`, `mut commands : Commands`, `assets : Res < GameAssets >`, `grid : Res < GridRes >`, `roster : Res < Roster >`, `windows : Query < & Window >`, `q_cam : Query < (& Camera , & GlobalTransform) , With < MainCamera > >`, `mut ctx : Ctx3d`

