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

**Parâmetros**: `mut commands : Commands`, `session : Res < Session >`, `args : Res < CliArgs >`, `mut net : ResMut < Net >`, `mut blobs : ResMut < Blobs >`, `mut images : ResMut < Assets < Image > >`, `mut map_state : ResMut < map :: MapState >`, `assets : Res < GameAssets >`, `grid : Res < grid :: GridRes >`, `roster : Res < Roster >`, `mut ctx : lowpoly :: Ctx3d`

## Implementações

### `impl Default for impl Default for ScreenInfo { fn default () -> Self { Self { width : 1366.0 , height : 840.0 , scale : 1.0 , auto_scale : true } } } . self_ty`

- `default`

### `impl Plugin for impl Plugin for GamePlugin { fn build (& self , app : & mut App) { app . init_resource :: < UiHovered > () . init_resource :: < ScreenInfo > () . init_resource :: < ActiveTool > () . init_resource :: < camera :: CamRig > () . init_resource :: < lowpoly :: Mats > () . init_resource :: < map :: DropMode > () . init_resource :: < map :: MapState > () . init_resource :: < grid :: GridRes > () . init_resource :: < terrain :: Terrain > () . init_resource :: < terrain :: TerrainRender > () . init_resource :: < tokens :: Selection > () . init_resource :: < tokens :: Dragging > () . init_resource :: < tokens :: TouchDrag > () ; # [cfg (target_os = "android")] app . init_resource :: < camera :: TouchState > () ; app . add_systems (Startup , (camera :: setup_camera , lowpoly :: setup_lowpoly , setup_lighting)) . add_systems (OnEnter (AppState :: InGame) , (hud :: setup_hud , game_init)) . add_systems (OnExit (AppState :: InGame) , (leave_game , reset_ui_hover)) . add_systems (First , screen_update) . add_systems (Update , (track_ui_hover , camera :: pan_zoom , # [cfg (target_os = "android")] camera :: touch_pan_zoom , camera :: apply_rig . after (camera :: pan_zoom) , grid :: draw_grid , grid :: grid_reflow , map :: file_drop , map :: sync_map , tokens :: token_interact , tokens :: token_y_follow . after (tokens :: token_interact) , tokens :: selection_visual , tokens :: delete_selected , # [cfg (target_os = "android")] tokens :: touch_interact , # [cfg (target_os = "android")] tokens :: touch_highlight , tokens :: resolve_pending_art , tokens :: refresh_ring_colors , terrain :: terrain_tool , terrain :: terrain_render . after (terrain :: terrain_tool) ,) . run_if (in_state (AppState :: InGame)) ,) . add_systems (Update , (sync :: handle_hello , sync :: handle_core , sync :: handle_tokens , sync :: assign_token_rx , hud :: toolbar_clicks , hud :: toolbar_visuals , hud :: roster_panel , hud :: status_label , hud :: hint_label , hud :: back_btn_click , hud :: scale_btn_click , hud :: assign_token_click ,) . run_if (in_state (AppState :: InGame)) . after (NetSet) ,) ; } } . self_ty`

- `build`

