# Crate: `tabletop`

```mermaid
graph TD
  tabletop["tabletop"]
  net["net"]
  tabletop --> net
  game_camera["camera"]
  tabletop --> game_camera
  game_lowpoly["lowpoly"]
  tabletop --> game_lowpoly
  game_terrain["terrain"]
  tabletop --> game_terrain
  game_hud["hud"]
  tabletop --> game_hud
  game_sync["sync"]
  tabletop --> game_sync
  lib["lib"]
  tabletop --> lib
  room_discovery["room_discovery"]
  tabletop --> room_discovery
  game_grid["grid"]
  tabletop --> game_grid
  game_mod["mod"]
  tabletop --> game_mod
  main["main"]
  tabletop --> main
  game_tokens["tokens"]
  tabletop --> game_tokens
  lobby["lobby"]
  tabletop --> lobby
  protocol["protocol"]
  tabletop --> protocol
  svg_assets["svg_assets"]
  tabletop --> svg_assets
  game_map["map"]
  tabletop --> game_map
```

## Módulos

### [`net`](modules/net) — `src/net.rs`

- **Resources**: Session, Net, Roster, Blobs
- **Events**: NetRx, PeerEvent
- **Structs**: `NetPlugin`, `NetSet`, `NetRx`, `PeerEvent`, `Session`, `Net`, `RosterEntry`, `Roster`, `Incoming`, `Blobs`

### [`camera`](modules/game__camera) — `src/game/camera.rs`

- **Resources**: CamRig, TouchState
- **Components**: MainCamera
- **Systems**: setup_camera
- **Structs**: `MainCamera`, `CamRig`, `TouchState`

### [`lowpoly`](modules/game__lowpoly) — `src/game/lowpoly.rs`

- **Resources**: LowPoly, Mats
- **Systems**: setup_lowpoly, spawn_tree
- **Structs**: `LowPoly`, `Mats`, `Ctx3d`

### [`terrain`](modules/game__terrain) — `src/game/terrain.rs`

- **Resources**: Terrain, TerrainRender
- **Systems**: terrain_render
- **Structs**: `Terrain`, `TerrainRender`
- **Enums**: `Op`

### [`hud`](modules/game__hud) — `src/game/hud.rs`

- **Components**: HudRoot, RosterPanel, RosterRow, StatusLabel, HintLabel, BackBtn, ScaleUpBtn, ScaleDownBtn, AssignTokenBtn
- **Systems**: tool_button, spawn_hud, setup_hud, scale_btn_click, roster_panel
- **Structs**: `HudRoot`, `RosterPanel`, `RosterRow`, `StatusLabel`, `HintLabel`, `BackBtn`, `ScaleUpBtn`, `ScaleDownBtn`, `AssignTokenBtn`
- **Enums**: `ToolBtn`

### [`sync`](modules/game__sync) — `src/game/sync.rs`

- **Systems**: handle_tokens

### [`lib`](modules/lib) — `src/lib.rs`

- **Resources**: CliArgs
- **Systems**: screenshot_hotkey, auto_shot_exit
- **Structs**: `CliArgs`
- **Enums**: `AppState`

### [`room_discovery`](modules/room_discovery) — `src/room_discovery.rs`

- **Structs**: `RoomEntry`

### [`grid`](modules/game__grid) — `src/game/grid.rs`

- **Resources**: GridRes
- **Systems**: draw_grid
- **Structs**: `GridRes`

### [`mod`](modules/game__mod) — `src/game/mod.rs`

- **Resources**: ScreenInfo, UiHovered
- **Systems**: setup_lighting, leave_game, game_init
- **Structs**: `ScreenInfo`, `UiHovered`, `GamePlugin`
- **Enums**: `ActiveTool`

### [`main`](modules/main) — `src/main.rs`


### [`tokens`](modules/game__tokens) — `src/game/tokens.rs`

- **Resources**: Selection, Dragging, TouchDrag
- **Components**: Token, PendingArt, OwnerRing, ArtDisc, SelRing
- **Systems**: spawn_token, delete_selected, resolve_pending_art
- **Structs**: `Token`, `PendingArt`, `OwnerRing`, `ArtDisc`, `SelRing`, `Selection`, `Dragging`, `TouchDrag`

### [`lobby`](modules/lobby) — `src/lobby.rs`

- **Resources**: RoomList, LobbyForm
- **Components**: LobbyRoot, NickField, CodeField, NickText, CodeText, StatusText, Swatch, CreateBtn, JoinBtn, RoomsPanel, RoomsContainer, RoomEntryBtn, RoomEmptyLabel
- **Systems**: setup_lobby, cleanup_lobby, start_session, lobby_auto, lobby_clicks, room_poll
- **Structs**: `RoomList`, `LobbyPlugin`, `LobbyForm`, `LobbyRoot`, `NickField`, `CodeField`, `NickText`, `CodeText`, `StatusText`, `Swatch`, `CreateBtn`, `JoinBtn`, `RoomsPanel`, `RoomsContainer`, `RoomEntryBtn`, `RoomEmptyLabel`
- **Enums**: `Focus`

### [`protocol`](modules/protocol) — `src/protocol.rs`

 # Módulo: protocol
 Define os tipos de dados e mensagens trocadas entre peers via WebRTC.
 Toda comunicação serializada com `bincode` usa os tipos e enum `Msg`
 declarados aqui. O GM é autoritativo: jogadores enviam `*Req`, GM
 valida e broadcast a versão final.

- **Structs**: `GridCfg`, `TerrainCell`, `TokenMeta`, `PlayerMeta`
- **Enums**: `GridKind`, `TokenArt`, `BlobKind`, `Msg`

### [`svg_assets`](modules/svg_assets) — `src/svg_assets.rs`

- **Resources**: GameAssets
- **Systems**: load_svgs
- **Structs**: `SvgAssetsPlugin`, `GameAssets`

### [`map`](modules/game__map) — `src/game/map.rs`

- **Resources**: MapState
- **Components**: MapGround
- **Systems**: sync_map, file_drop
- **Structs**: `MapState`, `MapGround`
- **Enums**: `DropMode`

