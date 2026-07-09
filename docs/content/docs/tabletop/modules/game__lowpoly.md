# `lowpoly`

**Path**: `src/game/lowpoly.rs`

## Resources (Bevy)

### `LowPoly`

| Campo | Tipo |
|-------|------|
| `cube` | `Handle < Mesh >` |
| `hex_prism` | `Handle < Mesh >` |
| `cylinder` | `Handle < Mesh >` |
| `cone` | `Handle < Mesh >` |

### `Mats`

| Campo | Tipo |
|-------|------|
| `terrain` | `HashMap < (u8 , i8) , Handle < StandardMaterial > >` |
| `rings` | `HashMap < u8 , Handle < StandardMaterial > >` |
| `art` | `HashMap < TokenArt , Handle < StandardMaterial > >` |
| `gold` | `Option < Handle < StandardMaterial > >` |
| `gray_ring` | `Option < Handle < StandardMaterial > >` |
| `pending` | `Option < Handle < StandardMaterial > >` |
| `trunk` | `Option < Handle < StandardMaterial > >` |
| `leaves` | `[Option < Handle < StandardMaterial > > ; 2]` |

## Structs

### `Ctx3d`

**Derives**: SystemParam

| Campo | Tipo |
|-------|------|
| `lp` | `Res < 'w , LowPoly >` |
| `mats` | `ResMut < 'w , Mats >` |
| `materials` | `ResMut < 'w , Assets < StandardMaterial > >` |
| `terrain` | `Res < 'w , Terrain >` |

## Funções

### `flat`

```rust
fn flat(color : Color) -> StandardMaterial
```

### `hex_prism_mesh`

```rust
fn hex_prism_mesh() -> Mesh
```

## Systems (Bevy)

### `setup_lowpoly`

**Parâmetros**: `mut commands : Commands`, `mut meshes : ResMut < Assets < Mesh > >`

### `spawn_tree`

**Parâmetros**: `parent : & mut ChildSpawnerCommands`, `lp : & LowPoly`, `mats : & mut Mats`, `materials : & mut Assets < StandardMaterial >`, `pos : Vec2`, `size : f32`, `variant : usize`

## Implementações

### `impl impl Mats { pub fn terrain (& mut self , materials : & mut Assets < StandardMaterial > , assets : & GameAssets , tex : u8 , elev : i8 ,) -> Handle < StandardMaterial > { if let Some (h) = self . terrain . get (& (tex , elev)) { return h . clone () ; } let f = (1.0 + elev as f32 * 0.09) . clamp (0.35 , 1.5) ; let m = if (tex as usize) < assets . textures . len () { StandardMaterial { base_color_texture : Some (assets . textures [tex as usize] . clone ()) , base_color : Color :: srgb (f , f , f * 0.97) , perceptual_roughness : 0.95 , reflectance : 0.08 , .. Default :: default () } } else if elev >= 0 { flat (Color :: srgb (0.72 * f , 0.68 * f , 0.58 * f)) } else { flat (Color :: srgb (0.20 , 0.16 , 0.26)) } ; let h = materials . add (m) ; self . terrain . insert ((tex , elev) , h . clone ()) ; h } pub fn ring (& mut self , materials : & mut Assets < StandardMaterial > , color_idx : u8) -> Handle < StandardMaterial > { if let Some (h) = self . rings . get (& color_idx) { return h . clone () ; } let h = materials . add (StandardMaterial { base_color : palette_color (color_idx) , perceptual_roughness : 0.55 , reflectance : 0.25 , .. Default :: default () }) ; self . rings . insert (color_idx , h . clone ()) ; h } pub fn gray_ring (& mut self , materials : & mut Assets < StandardMaterial >) -> Handle < StandardMaterial > { if let Some (h) = & self . gray_ring { return h . clone () ; } let h = materials . add (flat (Color :: srgb (0.55 , 0.55 , 0.58))) ; self . gray_ring = Some (h . clone ()) ; h } pub fn gold (& mut self , materials : & mut Assets < StandardMaterial >) -> Handle < StandardMaterial > { if let Some (h) = & self . gold { return h . clone () ; } let h = materials . add (StandardMaterial { base_color : Color :: srgb (0.95 , 0.80 , 0.22) , emissive : LinearRgba :: new (0.6 , 0.45 , 0.05 , 1.0) , perceptual_roughness : 0.4 , .. Default :: default () }) ; self . gold = Some (h . clone ()) ; h } # [doc = " Material da arte do token; `None` se o blob ainda não chegou (usar pending + PendingArt)."] pub fn art (& mut self , materials : & mut Assets < StandardMaterial > , assets : & GameAssets , blobs : & Blobs , art : TokenArt ,) -> Option < Handle < StandardMaterial > > { if let Some (h) = self . art . get (& art) { return Some (h . clone ()) ; } let img = match art { TokenArt :: BuiltIn (i) => assets . tokens_builtin [i as usize % assets . tokens_builtin . len ()] . clone () , TokenArt :: Blob (id) => blobs . images . get (& id) ? . clone () , } ; let h = materials . add (StandardMaterial { base_color_texture : Some (img) , perceptual_roughness : 0.8 , reflectance : 0.1 , .. Default :: default () }) ; self . art . insert (art , h . clone ()) ; Some (h) } pub fn pending (& mut self , materials : & mut Assets < StandardMaterial >) -> Handle < StandardMaterial > { if let Some (h) = & self . pending { return h . clone () ; } let h = materials . add (flat (Color :: srgb (0.35 , 0.33 , 0.40))) ; self . pending = Some (h . clone ()) ; h } pub fn trunk (& mut self , materials : & mut Assets < StandardMaterial >) -> Handle < StandardMaterial > { if let Some (h) = & self . trunk { return h . clone () ; } let h = materials . add (flat (Color :: srgb (0.42 , 0.29 , 0.18))) ; self . trunk = Some (h . clone ()) ; h } pub fn leaves (& mut self , materials : & mut Assets < StandardMaterial > , variant : usize) -> Handle < StandardMaterial > { let v = variant % 2 ; if let Some (h) = & self . leaves [v] { return h . clone () ; } let c = if v == 0 { Color :: srgb (0.25 , 0.48 , 0.20) } else { Color :: srgb (0.33 , 0.58 , 0.28) } ; let h = materials . add (flat (c)) ; self . leaves [v] = Some (h . clone ()) ; h } } . self_ty`

- `terrain`
- `ring`
- `gray_ring`
- `gold`
- `art`
- `pending`
- `trunk`
- `leaves`

## Constantes

| Nome | Tipo | Valor |
|------|------|-------|
| `BASE_CELL` | `# [doc = " Tokens e árvores são modelados para célula de 64 e escalados pelo Transform pai."] pub const BASE_CELL : f32 = 64.0 ; . ty` | `# [doc = " Tokens e árvores são modelados para célula de 64 e escalados pelo Transform pai."] pub const BASE_CELL : f32 = 64.0 ; . expr` |

