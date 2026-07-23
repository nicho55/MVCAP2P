# `terrain`

**Path**: `src/game/terrain.rs`

## Resources (Bevy)

### `Terrain`

| Campo | Tipo |
|-------|------|
| `cells` | `HashMap < Cell , TerrainCell >` |

### `TerrainRender`

| Campo | Tipo |
|-------|------|
| `ents` | `HashMap < Cell , Entity >` |
| `dirty` | `Vec < Cell >` |
| `full` | `bool` |

## Enums

### `Op`

| Variante | Campos |
|----------|--------|
| `Paint` | `u8` |
| `Erase` | `—` |
| `Elev` | `i8` |

## Funções

### `elev_height`

```rust
fn elev_height(cell : f32, elev : i8) -> f32
```

 Altura da coluna de uma célula. Elevação <= 0 vira um ladrilho fino

 (rebaixo é indicado por cor escura — não dá para cavar abaixo do plano do mapa).

### `cell_top`

```rust
fn cell_top(terrain : & Terrain, g : & GridCfg, cell : Cell) -> f32
```

 Altura do topo da célula (onde uma peça deve apoiar).

### `set_cell`

```rust
fn set_cell(terrain : & mut Terrain, render : & mut TerrainRender, cell : Cell, val : Option < TerrainCell >) -> bool
```

### `terrain_tool`

```rust
fn terrain_tool(buttons : Res < ButtonInput < MouseButton > >, mut touch_ev : MessageReader < TouchInput >, tool : Res < ActiveTool >, ui : Res < UiHovered >, drag : Res < TouchDrag >, session : Res < Session >, windows : Query < & Window >, q_cam : Query < (& Camera , & GlobalTransform) , With < MainCamera > >, grid : Res < GridRes >, mut terrain : ResMut < Terrain >, mut render : ResMut < TerrainRender >, mut net : ResMut < Net >, mut stroke : Local < HashSet < Cell > >) -> ()
```

## Systems (Bevy)

### `terrain_render`

 Materializa células como prismas low-poly (cubo no grid quadrado,

 prisma hexagonal no grid hex), com altura = elevação.

**Parâmetros**: `mut commands : Commands`, `mut render : ResMut < TerrainRender >`, `grid : Res < GridRes >`, `assets : Res < GameAssets >`, `mut ctx : Ctx3d`

