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

 Contexto agrupado para spawn de tokens/terreno 3D (evita estourar o limite de params).

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

 Prisma hexagonal flat-top (circumraio 1, altura 1, centrado na origem),

 vértices duplicados por face para flat shading.

## Systems (Bevy)

### `setup_lowpoly`

**Parâmetros**: `mut commands : Commands`, `mut meshes : ResMut < Assets < Mesh > >`

### `spawn_tree`

 Árvore low-poly: tronco + dois cones de copa. Filha da entidade do mapa.

**Parâmetros**: `parent : & mut ChildSpawnerCommands`, `lp : & LowPoly`, `mats : & mut Mats`, `materials : & mut Assets < StandardMaterial >`, `pos : Vec2`, `size : f32`, `variant : usize`

## Implementações

### `impl Mats`

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
| `BASE_CELL` | `f32` | `64.0` |

