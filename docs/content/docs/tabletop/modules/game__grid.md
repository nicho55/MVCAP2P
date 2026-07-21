# `grid`

**Path**: `src/game/grid.rs`

## Resources (Bevy)

### `GridRes`

## Funções

### `cell_center`

```rust
fn cell_center(g : & GridCfg, cell : Cell) -> Vec2
```

 Centro da célula em coordenadas de chão (x, z).

### `world_to_cell`

```rust
fn world_to_cell(g : & GridCfg, w : Vec2) -> Cell
```

### `axial_round`

```rust
fn axial_round(q : f32, r : f32) -> Cell
```

### `hex_corners`

```rust
fn hex_corners(g : & GridCfg, cell : Cell) -> [Vec2 ; 6]
```

### `v3`

```rust
fn v3(p : Vec2) -> Vec3
```

### `grid_reflow`

```rust
fn grid_reflow(grid : Res < GridRes >, mut trender : ResMut < TerrainRender >, mut q_tokens : Query < (& mut Transform , & Token) >) -> ()
```

 Quando o grid muda (tipo/tamanho de célula), reposiciona e reescala tokens

 e redesenha o terreno. Filhos dos tokens escalam junto com o pai.

## Systems (Bevy)

### `draw_grid`

 Desenha o grid no chão (XZ) em volta do foco da câmera, limitado ao mapa.

**Parâmetros**: `mut gizmos : Gizmos`, `grid : Res < GridRes >`, `rig : Res < CamRig >`, `map_state : Res < MapState >`, `settings : Res < super :: graphics :: GraphicsSettings >`

## Constantes

| Nome | Tipo | Valor |
|------|------|-------|
| `SQRT3` | `f32` | `1.732_050_8` |
| `GRID_Y` | `f32` | `0.45` |

