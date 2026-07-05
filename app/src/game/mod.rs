pub mod camera;
pub mod grid;
pub mod hud;
pub mod lowpoly;
pub mod map;
pub mod sync;
pub mod terrain;
pub mod tokens;

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use crate::net::{Blobs, Net, NetSet, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::GameAssets;
use crate::{AppState, CliArgs};

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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiHovered>()
            .init_resource::<ActiveTool>()
            .init_resource::<camera::CamRig>()
            .init_resource::<lowpoly::Mats>()
            .init_resource::<map::DropMode>()
            .init_resource::<map::MapState>()
            .init_resource::<grid::GridRes>()
            .init_resource::<terrain::Terrain>()
            .init_resource::<terrain::TerrainRender>()
            .init_resource::<tokens::Selection>()
            .init_resource::<tokens::Dragging>()
            .add_systems(Startup, (camera::setup_camera, lowpoly::setup_lowpoly, setup_lighting))
            .add_systems(OnEnter(AppState::InGame), (hud::setup_hud, game_init))
            .add_systems(
                Update,
                (
                    track_ui_hover,
                    camera::pan_zoom,
                    camera::apply_rig.after(camera::pan_zoom),
                    grid::draw_grid,
                    grid::grid_reflow,
                    map::file_drop,
                    map::sync_map,
                    tokens::token_interact,
                    tokens::token_y_follow.after(tokens::token_interact),
                    tokens::selection_visual,
                    tokens::delete_selected,
                    tokens::resolve_pending_art,
                    tokens::refresh_ring_colors,
                    terrain::terrain_tool,
                    terrain::terrain_render.after(terrain::terrain_tool),
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    sync::handle_hello,
                    sync::handle_core,
                    sync::handle_tokens,
                    hud::toolbar_clicks,
                    hud::toolbar_visuals,
                    hud::roster_panel,
                    hud::status_label,
                    hud::hint_label,
                )
                    .run_if(in_state(AppState::InGame))
                    .after(NetSet),
            );
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            color: Color::srgb(1.0, 0.97, 0.90),
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
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.85, 0.88, 1.0),
        brightness: 350.0,
        ..default()
    });
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
            Ok(bytes) => map::import_map_bytes(bytes, &mut blobs, &mut images, &mut net, &mut map_state),
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
            tokens::spawn_token(&mut commands, meta.clone(), &assets, &blobs, &grid.0, &roster, &mut ctx);
            net.broadcast(&Msg::SpawnToken(meta));
        }
        info!("tokens de demonstração criados");
    }
}
