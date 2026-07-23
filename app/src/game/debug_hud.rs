use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use super::terrain::TerrainRender;
use super::ScreenInfo;
use crate::net::{Net, Roster, Session};
use crate::svg_assets::{tfont, GameAssets};

const PANEL: Color = Color::srgba(0.05, 0.04, 0.08, 0.85);
const TEXT: Color = Color::srgb(0.70, 0.90, 0.55);

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

#[derive(Component)]
pub struct DebugHudRoot;

#[derive(Component)]
pub struct DebugText;

pub fn spawn_debug_hud(
    mut commands: Commands,
    session: Res<Session>,
    assets: Res<GameAssets>,
    si: Res<ScreenInfo>,
) {
    if !session.is_test_room {
        return;
    }
    commands
        .spawn((
            DebugHudRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(sz(8.0, &si)),
                left: Val::Px(sz(8.0, &si)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(sz(6.0, &si))),
                row_gap: Val::Px(sz(2.0, &si)),
                min_width: Val::Px(sz(220.0, &si)),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("DEBUG"),
                tfont(&assets, sz(12.0, &si)),
                TextColor(Color::srgb(0.83, 0.69, 0.22)),
            ));
            root.spawn((
                DebugText,
                Text::new(""),
                tfont(&assets, sz(11.0, &si)),
                TextColor(TEXT),
            ));
        });
}

pub fn despawn_debug_hud(mut commands: Commands, q: Query<Entity, With<DebugHudRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn update_debug_hud(
    session: Option<Res<Session>>,
    diagnostics: Res<DiagnosticsStore>,
    net: Option<Res<Net>>,
    roster: Option<Res<Roster>>,
    render: Option<Res<TerrainRender>>,
    mut q: Query<&mut Text, With<DebugText>>,
    q_all: Query<Entity>,
) {
    let Some(sess) = session else { return };
    if !sess.is_test_room {
        return;
    }
    let Ok(mut text) = q.single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let entities = q_all.iter().count();

    let chunks = render.as_ref().map(|r| r.ents.len()).unwrap_or(0);

    let peers = net
        .as_ref()
        .map(|n| n.peers().len())
        .unwrap_or(0);

    let players = roster.as_ref().map(|r| r.list.len()).unwrap_or(0);

    text.0 = format!(
        "FPS: {fps:.0}  ({frame_ms:.1}ms)\n\
         Entities: {entities}\n\
         Terrain cells: {chunks}\n\
         Peers: {peers}  Players: {players}"
    );
}
