use bevy::prelude::*;

use crate::game::ScreenInfo;
use crate::svg_assets::{tfont, GameAssets};

#[derive(Component)]
pub struct UiLayer;

#[derive(Component)]
pub struct NotificationArea;

#[derive(Component)]
pub struct ConnIndicator;

pub struct UiLayerPlugin;

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui_layer)
            .add_systems(Update, update_conn_indicator);
    }
}

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

fn spawn_ui_layer(mut commands: Commands, assets: Res<GameAssets>, si: Res<ScreenInfo>) {
    commands
        .spawn((
            UiLayer,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            Visibility::Inherited,
            // High z-index so notifications render above everything
            ZIndex(100),
        ))
        .with_children(|root| {
            root.spawn((
                ConnIndicator,
                Text::new(""),
                tfont(&assets, sz(11.0, &si)),
                TextColor(Color::srgb(0.50, 0.48, 0.55)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(sz(4.0, &si)),
                    left: Val::Px(sz(4.0, &si)),
                    ..default()
                },
            ));

            root.spawn((
                NotificationArea,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(sz(4.0, &si)),
                    left: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(sz(4.0, &si)),
                    ..default()
                },
            ));
        });
}

fn update_conn_indicator(
    state: Res<State<crate::AppState>>,
    net: Option<Res<crate::net::Net>>,
    mut q: Query<&mut Text, With<ConnIndicator>>,
) {
    let Ok(mut text) = q.single_mut() else {
        return;
    };
    let label = match state.get() {
        crate::AppState::Boot => "inicializando...",
        crate::AppState::Lobby => "lobby",
        crate::AppState::InGame => {
            if net.as_ref().is_some_and(|n| n.gm_peer.is_some()) {
                "conectado"
            } else {
                "conectando..."
            }
        }
    };
    if text.0 != label {
        text.0 = label.to_string();
    }
}
