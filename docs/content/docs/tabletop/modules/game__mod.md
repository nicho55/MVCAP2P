# `mod`

**Path**: `src/game/mod.rs`

## Resources (Bevy)

### `ScreenInfo`

| Campo | Tipo |
|-------|------|
| `width` | `f32` |
| `height` | `f32` |
| `scale` | `f32` |
| `auto_scale` | `bool` |

### `UiHovered`

## Structs

### `GamePlugin`

**Derives**: 

## Enums

### `ActiveTool`

**Derives**: Resource, Default, Clone, Copy, PartialEq

| Variante | Campos |
|----------|--------|
| `Select` | `—` |
| `Paint` | `u8` |
| `Erase` | `—` |
| `ElevUp` | `—` |
| `ElevDown` | `—` |

## Funções

### `screen_update`

```rust
fn screen_update(mut si : ResMut < ScreenInfo >, q_win : Query < & Window >) -> ()
```

 Atualiza `width`/`height` do ScreenInfo. A escala automática só roda

 na primeira detecção da janela; depois o usuário controla com A+/A-.

### `reset_ui_hover`

```rust
fn reset_ui_hover(mut h : ResMut < UiHovered >) -> ()
```

### `track_ui_hover`

```rust
fn track_ui_hover(q : Query < & Interaction >, mut h : ResMut < UiHovered >) -> ()
```

## Systems (Bevy)

### `setup_lighting`

**Parâmetros**: `mut commands : Commands`

### `leave_game`

**Parâmetros**: `mut commands : Commands`, `mut net : ResMut < Net >`, `session : Option < Res < Session > >`, `mut render : ResMut < terrain :: TerrainRender >`, `q_hud : Query < Entity , With < hud :: HudRoot > >`, `q_ground : Query < Entity , With < map :: MapGround > >`, `q_tokens : Query < Entity , With < tokens :: Token > >`

### `game_init`

 GM: carrega mapa via --map e tokens de demonstração via --demo.

**Parâmetros**: `mut commands : Commands`, `session : Res < Session >`, `args : Res < CliArgs >`, `mut net : ResMut < Net >`, `mut blobs : ResMut < Blobs >`, `mut images : ResMut < Assets < Image > >`, `mut map_state : ResMut < map :: MapState >`, `assets : Res < GameAssets >`, `grid : Res < grid :: GridRes >`, `roster : Res < Roster >`, `mut ctx : lowpoly :: Ctx3d`

## Implementações

### `impl Default for ScreenInfo`

- `default`

### `impl Plugin for GamePlugin`

- `build`

