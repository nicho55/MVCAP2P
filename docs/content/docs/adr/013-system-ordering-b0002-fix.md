# ADR-013: System Ordering – B0002 Crash Fix

## Status

Accepted

## Context

O APK debug crashava no Android logo ao entrar na sala de teste com `error[B0002]`: `ResMut<X> in system <Y> conflicts with a previous Res<X> access`. A causa raiz era a ausência de ordenação explícita entre systems que acessam os mesmos resources no `Update` schedule.

Três problemas distintos foram identificados:

### 1. Flat Tuples no `game/mod.rs`

Todos os systems do GamePlugin eram registrados em flat tuples sem `.chain()`:
```rust
add_systems(Update, (
    sync::handle_hello,
    hud::toolbar_clicks,
    camera::pan_zoom,
    tokens::token_interact,
    map::sync_map,
))
```
Bevy panics ao inicializar o schedule porque não consegue determinar a ordem de execução entre systems que escrevem no mesmo resource.

### 2. `ScreenInfo` escrito por múltiplos systems

- `screen_update` (em `First`) escrevia `ResMut<ScreenInfo>`
- `hud::scale_btn_click` (em `HudWriteSet`) também escrevia `ResMut<ScreenInfo>`

### 3. VirtualJoystickPlugin sem ordenação inter-plugin

O `VirtualJoystickPlugin` registrava systems no Update sem nenhuma restrição de ordem em relação aos systems do GamePlugin:
- `joystick_apply` escrevia `ResMut<CamRig>` → conflita com `camera::apply_rig` (lê `Res<CamRig>`)
- `joystick_input` lia `Res<ScreenInfo>` → conflita com `hud::scale_btn_click` (escreve `ResMut<ScreenInfo>`)

## Decision

### System Sets com ordenação em cascata

Criados dois `SystemSet` em `game/mod.rs`:

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct SyncSet;    // Network sync systems

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HudWriteSet;  // HUD write systems
```

Ordem global: `SyncSet → HudWriteSet → systems de leitura`

### Reorganização em 4 fases (game/mod.rs)

1. **SyncSet** — `handle_hello`, `handle_core`, `handle_tokens`, `assign_token_rx`
2. **HudWriteSet** — `toolbar_clicks`, `delete_btn_click`, `assign_token_click`, `scale_btn_click`, `back_btn_click`, `gfx_open_click`, `gfx_toggle_click`
3. **Leitura visual** — `toolbar_visuals`, `hint_label`, `delete_btn_visibility`, `roster_panel`, `status_label`, `gfx_panel_visuals` — todos `.after(HudWriteSet)`
4. **Gameplay** — `track_ui_hover`, `camera::pan_zoom`, `camera::apply_rig`, `map::sync_map`, `tokens::token_interact`, `terrain::terrain_tool`, `grid::*`

### ScreenInfo reativo

`screen_update` movido para `First` e recalcula a cada frame quando `auto_scale == true`. `scale_btn_click` foi removido (escala automática substitui ajuste manual).

### VirtualJoystickPlugin com ordenação inter-plugin

```rust
// game/mod.rs
camera::apply_rig
    .after(camera::pan_zoom)
    .after(virtual_joystick::joystick_apply),

// joystick systems ordenados contra HudWriteSet
joystick_apply.after(camera::pan_zoom).after(HudWriteSet),
joystick_input.after(HudWriteSet),
```

`joystick_apply` e a stub desktop foram tornadas `pub(crate)` para permitir referência cruzada.

### Plataforma específica

`terrain::terrain_tool` tem ordenação condicional:
- Desktop: `.after(track_ui_hover).after(tokens::delete_selected).after(HudWriteSet)`
- Android: `.after(track_ui_hover).after(tokens::touch_interact).after(HudWriteSet)`

### ChunkRender: conflito entre 3 systems pós-merge

Após os PRs #36 (graphics) e #37 (debug_hud) serem mergeados, surgiu um novo B0002 no Android: três systems acessam `ChunkRender` após `HudWriteSet` sem ordenação entre si:

| System | Acesso | Bloco |
|---|---|---|
| `graphics::apply_graphics` | `ResMut<ChunkRender>` | `.after(HudWriteSet)` |
| `terrain::chunk_render_system` | `ResMut<ChunkRender>` | `.after(HudWriteSet)` |
| `debug_hud::update_debug_hud` | `Res<ChunkRender>` | `.after(HudWriteSet)` |

### Auditoria completa — conflitos adicionais

A correção do ChunkRender expôs que o B0002 reaparecia com conflitos em outros resources. Auditoria completa identificou:

| Resource | System A (acesso) | System B (acesso) | Tipo |
|---|---|---|---|
| Mats / Assets<StdMat> (via Ctx3d) | sync_map (W) | resolve_pending_art (W) | W/W |
| Mats / Assets<StdMat> (via Ctx3d) | resolve_pending_art (W) | refresh_ring_colors (W) | W/W |
| Mats / Assets<StdMat> (via Ctx3d) | refresh_ring_colors (W) | chunk_render_system (W) | W/W |
| CamRig | pan_zoom (W) | chunk_render_system (R) | W/R |
| CamRig | touch_pan_zoom (W) | apply_rig (R) | W/R |
| CamRig | touch_pan_zoom (W) | joystick_apply (W) | W/W |
| UiHovered | track_ui_hover (W) | pan_zoom (R) | W/R |
| Terrain | terrain_tool (W) | resolve_pending_art (R via Ctx3d) | W/R |
| Terrain | terrain_tool (W) | token_y_follow (R) | W/R |
| Selection (B0001) | delete_btn_click | (mesmo system Res+ResMut) | aliasing |
| Ctx3d | handle_tokens (W) | assign_token_rx (W) | W/W |

### Cadeia de ordenação final (gameplay block)

```
SyncSet:
  handle_hello → handle_core → handle_tokens → assign_token_rx

Gameplay:
  track_ui_hover → pan_zoom → touch_pan_zoom* → apply_rig
                                                  ↓
  sync_map → file_drop ─┬→ token_interact → delete_selected → terrain_tool
                         │                                      ↓
                         └→ resolve_pending_art → refresh_ring_colors
                                                      ↓
  grid_reflow ────────────────────────────→ chunk_render_system
                                           (after: terrain_tool, apply_graphics,
                                            apply_rig, grid_reflow, refresh_ring_colors)
                                                      ↓
                                            debug_hud::update_debug_hud
```

*`touch_pan_zoom` e `joystick_apply` são Android-only.

## Consequences

### Positivas
- Crash B0002 eliminado — todas as dependências de resource estão explícitas
- Sistemas executam em ordem determinística e previsível
- Adição de novos systems não causa conflitos desde que respeitem os sets
- `joystick_apply` e `apply_rig` não competem por `CamRig` na mesma frame

### Negativas
- Ordenação explícita é verbosa — cada novo system precisa de `.after()`
- `pub(crate)` em `joystick_apply` expõe detalhe interno

### Neutras
- `ScreenInfo` recalcula em `First` — custo desprezível (uma vez por frame)
- `HudWriteSet` é `pub(crate)` para permitir uso por plugins externos
