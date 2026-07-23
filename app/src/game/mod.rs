pub mod camera;
pub mod graphics;
pub mod grid;
pub mod hud;
pub mod lowpoly;
pub mod map;
pub mod ruler;
pub mod sync;
pub mod terrain;
pub mod tokens;
pub mod virtual_joystick;

use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use crate::net::{Blobs, Net, NetSet, Roster, Session};
use crate::protocol::*;
use crate::room_discovery;
use crate::svg_assets::GameAssets;
use crate::{AppState, CliArgs};

#[derive(Resource)]
pub struct ScreenInfo {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    /// Quando `true` a escala é recalculada automaticamente a cada mudança
    /// de tamanho da janela (incluindo rotação de tela no Android). O botão
    /// A+/A- desativa a escala automática e assume controle manual.
    pub auto_scale: bool,
    /// Largura de referência (desktop) usada no cálculo da escala automática.
    ref_w: f32,
    /// Altura de referência (mobile) usada no cálculo da escala automática.
    ref_h: f32,
}

impl Default for ScreenInfo {
    fn default() -> Self {
        Self {
            width: 1366.0,
            height: 840.0,
            scale: 1.0,
            auto_scale: true,
            ref_w: 900.0,
            ref_h: 430.0,
        }
    }
}

/// Recalcula `width`/`height` e, quando `auto_scale` está ativo, a escala
/// responsiva. Roda a cada frame (em `First`) para reagir a mudanças de
/// tamanho/rotação da janela em tempo real.
fn screen_update(
    mut si: ResMut<ScreenInfo>,
    q_win: Query<&Window>,
    device: Res<crate::DeviceProfile>,
) {
    let Ok(win) = q_win.single() else { return };
    let w = win.resolution.width();
    let h = win.resolution.height();
    if (w - si.width).abs() > 1.0 || (h - si.height).abs() > 1.0 {
        si.width = w;
        si.height = h;
    }
    if si.auto_scale {
        si.scale = if device.is_mobile() {
            (h / si.ref_h).clamp(0.85, 1.6)
        } else {
            (w / si.ref_w).clamp(0.75, 2.0)
        };
    }
}

#[derive(Resource, Default)]
pub struct UiHovered(pub bool);

#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub enum ActiveTool {
    #[default]
    Select,
    Paint(u8),
    Erase,
    ElevUp,
    ElevDown,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct SyncSet;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HudWriteSet;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiHovered>()
            .init_resource::<ScreenInfo>()
            .init_resource::<ActiveTool>()
            .init_resource::<camera::CamRig>()
            .init_resource::<lowpoly::Mats>()
            .init_resource::<map::DropMode>()
            .init_resource::<map::MapState>()
            .init_resource::<grid::GridRes>()
            .init_resource::<terrain::Terrain>()
            .init_resource::<terrain::ChunkRender>()
            .init_resource::<tokens::Selection>()
            .init_resource::<tokens::Dragging>()
            .init_resource::<tokens::TouchDrag>()
            .init_resource::<graphics::GraphicsSettings>()
            .add_plugins(virtual_joystick::VirtualJoystickPlugin);
        #[cfg(target_os = "android")]
        app.init_resource::<camera::TouchState>();
        app.add_systems(
            Startup,
            (camera::setup_camera, lowpoly::setup_lowpoly, setup_lighting),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            (hud::setup_hud, game_init, graphics::spawn_gfx_ui),
        )
        .add_systems(
            OnExit(AppState::InGame),
            (leave_game, reset_ui_hover, graphics::despawn_gfx_ui),
        )
        .add_systems(First, screen_update)
        .configure_sets(Update, (SyncSet, HudWriteSet).chain())
        .add_systems(Update, graphics::apply_graphics.after(HudWriteSet))
        .add_systems(
            Update,
            (
                sync::handle_hello,
                sync::handle_core.after(sync::handle_hello),
                sync::handle_tokens.after(sync::handle_core),
                sync::assign_token_rx.after(sync::handle_core),
            )
                .in_set(SyncSet)
                .run_if(in_state(AppState::InGame))
                .after(NetSet),
        )
        .add_systems(
            Update,
            (
                hud::toolbar_clicks.after(SyncSet),
                hud::delete_btn_click.after(hud::toolbar_clicks),
                hud::assign_token_click.after(hud::delete_btn_click),
                hud::scale_btn_click.after(SyncSet),
                hud::back_btn_click,
                graphics::gfx_open_click,
                graphics::gfx_toggle_click.after(SyncSet),
            )
                .in_set(HudWriteSet)
                .run_if(in_state(AppState::InGame))
                .after(NetSet),
        )
        .add_systems(
            Update,
            (
                hud::toolbar_visuals.after(hud::toolbar_clicks),
                hud::hint_label.after(hud::toolbar_clicks),
                hud::delete_btn_visibility.after(hud::delete_btn_click),
                hud::roster_panel
                    .after(hud::scale_btn_click)
                    .after(hud::delete_btn_click)
                    .after(SyncSet),
                hud::status_label
                    .after(hud::assign_token_click)
                    .after(SyncSet),
                graphics::gfx_panel_visuals.after(graphics::gfx_toggle_click),
            )
                .run_if(in_state(AppState::InGame))
                .after(HudWriteSet)
                .after(NetSet),
        )
        .add_systems(
            Update,
            (
                track_ui_hover,
                camera::pan_zoom,
                #[cfg(target_os = "android")]
                camera::touch_pan_zoom.after(camera::pan_zoom),
                camera::apply_rig
                    .after(camera::pan_zoom)
                    .after(virtual_joystick::joystick_apply),
                map::sync_map.after(SyncSet),
                map::file_drop.after(map::sync_map).after(HudWriteSet),
                tokens::token_interact
                    .after(track_ui_hover)
                    .after(map::file_drop)
                    .after(HudWriteSet),
                tokens::token_y_follow.after(tokens::token_interact),
                tokens::delete_selected.after(tokens::token_interact),
                tokens::selection_visual
                    .after(tokens::token_interact)
                    .after(tokens::delete_selected),
                #[cfg(target_os = "android")]
                tokens::touch_interact
                    .after(track_ui_hover)
                    .after(tokens::delete_selected)
                    .after(HudWriteSet),
                #[cfg(target_os = "android")]
                tokens::touch_highlight,
                tokens::resolve_pending_art,
                tokens::refresh_ring_colors,
                #[cfg(not(target_os = "android"))]
                terrain::terrain_tool
                    .after(track_ui_hover)
                    .after(tokens::delete_selected)
                    .after(HudWriteSet),
                #[cfg(target_os = "android")]
                terrain::terrain_tool
                    .after(track_ui_hover)
                    .after(tokens::touch_interact)
                    .after(HudWriteSet),
                terrain::chunk_render_system.after(terrain::terrain_tool),
                grid::grid_reflow
                    .after(terrain::terrain_tool)
                    .after(SyncSet)
                    .after(HudWriteSet),
                grid::draw_grid
                    .after(map::sync_map)
                    .after(map::file_drop)
                    .after(camera::pan_zoom),
            )
                .run_if(in_state(AppState::InGame))
                .after(HudWriteSet)
                .after(NetSet),
        );
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            color: Color::srgb(1.0, 0.97, 0.90),
            // Estado inicial; GraphicsSettings::apply_graphics ajusta em runtime
            // (desligado por padrão no Android).
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.7, -0.95, 0.0)),
        CascadeShadowConfigBuilder {
            maximum_distance: 6000.0,
            first_cascade_far_bound: 1200.0,
            ..default()
        }
        .build(),
    ));
}

fn leave_game(
    mut commands: Commands,
    mut net: ResMut<Net>,
    session: Option<Res<Session>>,
    mut render: ResMut<terrain::ChunkRender>,
    q_hud: Query<Entity, With<hud::HudRoot>>,
    q_ground: Query<Entity, With<map::MapGround>>,
    q_tokens: Query<Entity, With<tokens::Token>>,
) {
    let code = session.as_ref().map(|s| s.code.clone());
    let is_gm = session.as_ref().map(|s| s.me.is_gm).unwrap_or(false);
    net.disconnect();
    render.meshes.clear();
    render.dirty.clear();
    for e in q_hud.iter().chain(q_ground.iter()).chain(q_tokens.iter()) {
        commands.entity(e).despawn();
    }
    if is_gm {
        if let Some(code) = code {
            std::thread::spawn(move || {
                let _ = room_discovery::delete_room(code.as_str());
            });
        }
    }
}

fn reset_ui_hover(mut h: ResMut<UiHovered>) {
    h.0 = false;
}

fn track_ui_hover(q: Query<&Interaction>, mut h: ResMut<UiHovered>) {
    let v = q.iter().any(|i| !matches!(i, Interaction::None));
    if h.0 != v {
        h.0 = v;
    }
}

/// GM: carrega mapa via --map e tokens de demonstração via --demo.
fn game_init(
    mut commands: Commands,
    session: Res<Session>,
    args: Res<CliArgs>,
    mut net: ResMut<Net>,
    mut blobs: ResMut<Blobs>,
    mut images: ResMut<Assets<Image>>,
    mut map_state: ResMut<map::MapState>,
    assets: Res<GameAssets>,
    grid: Res<grid::GridRes>,
    roster: Res<Roster>,
    mut ctx: lowpoly::Ctx3d,
) {
    if !session.me.is_gm {
        return;
    }
    if let Some(path) = &args.map {
        match std::fs::read(path) {
            Ok(bytes) => {
                map::import_map_bytes(bytes, &mut blobs, &mut images, &mut net, &mut map_state)
            }
            Err(e) => warn!("falha lendo mapa {path}: {e}"),
        }
    }
    if args.demo {
        for (i, cell) in [(0, 0), (2, 1), (-2, 0), (1, -2)].into_iter().enumerate() {
            let meta = TokenMeta {
                id: rand::random(),
                owner: session.me.uuid,
                art: TokenArt::BuiltIn(i as u8),
                cell,
            };
            tokens::spawn_token(
                &mut commands,
                meta.clone(),
                &assets,
                &blobs,
                &grid.0,
                &roster,
                &mut ctx,
            );
            net.broadcast(&Msg::SpawnToken(meta));
        }
        info!("tokens de demonstração criados");
    }
}
