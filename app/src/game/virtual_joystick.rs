//! Joystick virtual (#24) — componente **(3)** do layout: um stick no **inferior
//! direito** que move o **token selecionado** pela grid (não a câmera). A câmera
//! continua no toque livre (`camera::touch_pan_zoom`), que ignora o dedo do
//! joystick. Segue os mockups (`docs/layouts/mockup-landscape.svg`).
//!
//! Prioridade de toque: botão/painel (via `UiHovered`) > joystick (dentro do
//! círculo) > câmera (área livre). O movimento respeita a grid: o stick define
//! uma direção e o token anda célula a célula naquela direção (hex ou quadrada).
//!
//! Android-only: no desktop o token se move pelo mouse (`tokens::token_interact`).

use bevy::prelude::*;

use crate::AppState;

#[cfg(target_os = "android")]
use super::camera::CamRig;
#[cfg(target_os = "android")]
use super::grid;
#[cfg(target_os = "android")]
use super::grid::GridRes;
#[cfg(target_os = "android")]
use super::tokens::{Selection, Token};
#[cfg(target_os = "android")]
use super::HudWriteSet;
#[cfg(target_os = "android")]
use super::ScreenInfo;
#[cfg(target_os = "android")]
use super::UiHovered;
#[cfg(target_os = "android")]
use crate::net::{Net, Session};
#[cfg(target_os = "android")]
use crate::protocol::*;
#[cfg(target_os = "android")]
use bevy::input::touch::TouchPhase;

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
const RING_BG: Color = Color::srgba(0.15, 0.14, 0.20, 0.45);
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
const THUMB_COLOR: Color = Color::srgba(0.83, 0.69, 0.22, 0.55);
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
const ACTIVE_BORDER: Color = Color::srgba(0.83, 0.69, 0.22, 0.65);
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
const INACTIVE_BORDER: Color = Color::srgba(0.5, 0.5, 0.5, 0.25);

/// Direção do stick, normalizada em [-1, 1]. Zero = repousando.
#[derive(Resource, Default)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub struct JoystickState {
    pub dir: Vec2,
}

/// Dedo que está segurando o stick — a câmera ignora este toque.
#[cfg(target_os = "android")]
#[derive(Resource, Default)]
pub struct JoystickTouch(pub Option<u64>);

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct JoystickRoot;

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct Stick;

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct Thumb;

#[cfg(target_os = "android")]
#[derive(Resource)]
struct StickGeometry {
    center: Vec2,
    radius: f32,
    thumb_diameter: f32,
}

#[cfg(target_os = "android")]
fn spawn_joystick(
    mut commands: Commands,
    screen_info: Res<ScreenInfo>,
    existing: Query<Entity, With<JoystickRoot>>,
) {
    if !existing.is_empty() {
        return;
    }

    let diameter = 120.0 * screen_info.scale;
    let radius = diameter / 2.0;
    let margin = 40.0 * screen_info.scale;
    let thumb_diameter = diameter * 0.4;

    let center_x = screen_info.width - margin - radius;
    let center_y = screen_info.height - margin - radius;

    commands.insert_resource(StickGeometry {
        center: Vec2::new(center_x, center_y),
        radius,
        thumb_diameter,
    });

    let thumb_offset = radius - thumb_diameter / 2.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(screen_info.width),
                height: Val::Px(screen_info.height),
                ..default()
            },
            JoystickRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(center_x - radius),
                        top: Val::Px(center_y - radius),
                        width: Val::Px(diameter),
                        height: Val::Px(diameter),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(radius)),
                        ..default()
                    },
                    BackgroundColor(RING_BG),
                    BorderColor::all(INACTIVE_BORDER),
                    Stick,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(thumb_offset),
                            top: Val::Px(thumb_offset),
                            width: Val::Px(thumb_diameter),
                            height: Val::Px(thumb_diameter),
                            border_radius: BorderRadius::all(Val::Px(thumb_diameter / 2.0)),
                            ..default()
                        },
                        BackgroundColor(THUMB_COLOR),
                        Thumb,
                    ));
                });
        });
}

#[cfg(target_os = "android")]
fn joystick_input(
    mut touch_events: MessageReader<TouchInput>,
    mut state: ResMut<JoystickState>,
    mut touch: ResMut<JoystickTouch>,
    geo: Res<StickGeometry>,
    ui: Res<UiHovered>,
) {
    for event in touch_events.read() {
        let pos = event.position;
        match event.phase {
            TouchPhase::Started => {
                // Botão/painel tem prioridade; só captura dentro do círculo.
                if touch.0.is_none() && !ui.0 && pos.distance(geo.center) <= geo.radius {
                    touch.0 = Some(event.id);
                    clamp_to_stick(&mut state.dir, pos, geo.center, geo.radius);
                }
            }
            TouchPhase::Moved => {
                if touch.0 == Some(event.id) {
                    clamp_to_stick(&mut state.dir, pos, geo.center, geo.radius);
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if touch.0 == Some(event.id) {
                    touch.0 = None;
                    state.dir = Vec2::ZERO;
                }
            }
        }
    }
}

#[cfg(target_os = "android")]
fn clamp_to_stick(out: &mut Vec2, touch_pos: Vec2, center: Vec2, radius: f32) {
    let delta = touch_pos - center;
    let len = delta.length();
    *out = if len > radius {
        delta / len
    } else {
        delta / radius
    };
}

#[cfg(target_os = "android")]
fn update_thumb_position(
    state: Res<JoystickState>,
    geo: Res<StickGeometry>,
    mut thumb_q: Query<&mut Node, With<Thumb>>,
) {
    let Ok(mut node) = thumb_q.single_mut() else {
        return;
    };
    let half = geo.thumb_diameter / 2.0;
    let offset = state.dir * geo.radius;
    node.left = Val::Px(geo.radius + offset.x - half);
    node.top = Val::Px(geo.radius + offset.y - half);
}

/// Realce do anel: dourado quando há token selecionado, apagado quando não.
#[cfg(target_os = "android")]
fn joystick_feedback(
    sel: Res<Selection>,
    added: Query<(), Added<Stick>>,
    mut q: Query<&mut BorderColor, With<Stick>>,
) {
    if !sel.is_changed() && added.is_empty() {
        return;
    }
    let color = if sel.0.is_some() {
        ACTIVE_BORDER
    } else {
        INACTIVE_BORDER
    };
    if let Ok(mut border) = q.single_mut() {
        *border = BorderColor::all(color);
    }
}

/// Anda o token selecionado célula a célula na direção do stick (respeita a grid).
#[cfg(target_os = "android")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn joystick_move_token(
    state: Res<JoystickState>,
    sel: Res<Selection>,
    grid_res: Res<GridRes>,
    rig: Res<CamRig>,
    time: Res<Time>,
    mut step_timer: Local<f32>,
    mut net: ResMut<Net>,
    session: Res<Session>,
    mut q_tokens: Query<(&mut Transform, &mut Token)>,
) {
    const DEADZONE: f32 = 0.35;
    const STEP_SECS: f32 = 0.16;

    if state.dir.length() < DEADZONE {
        *step_timer = STEP_SECS; // pronto para andar assim que o stick mover
        return;
    }
    let Some(id) = sel.0 else { return };

    *step_timer += time.delta_secs();
    if *step_timer < STEP_SECS {
        return;
    }
    *step_timer = 0.0;

    // Direção da tela → mundo, usando o yaw da câmera (tela-cima = frente).
    let yaw_rot = Quat::from_rotation_y(rig.yaw);
    let right = yaw_rot * Vec3::X;
    let fwd = yaw_rot * Vec3::NEG_Z;
    let world = right * state.dir.x - fwd * state.dir.y;
    let world = Vec2::new(world.x, world.z);
    if world.length() < 0.01 {
        return;
    }
    let g = &grid_res.0;

    let Some((mut tf, mut tok)) = q_tokens.iter_mut().find(|(_, t)| t.meta.id == id) else {
        return;
    };
    let here = grid::cell_center(g, tok.meta.cell);
    let target = here + world.normalize() * g.cell;
    let cell = grid::world_to_cell(g, target);
    if cell == tok.meta.cell {
        return;
    }
    tok.meta.cell = cell;
    let c = grid::cell_center(g, cell);
    tf.translation.x = c.x;
    tf.translation.z = c.y;
    if session.me.is_gm {
        net.broadcast(&Msg::TokenMoved { id, cell });
    } else {
        net.send_gm(&Msg::MoveTokenReq { id, cell });
    }
}

#[cfg(target_os = "android")]
fn despawn_joystick(
    mut commands: Commands,
    query: Query<Entity, With<JoystickRoot>>,
    mut state: ResMut<JoystickState>,
    mut touch: ResMut<JoystickTouch>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<StickGeometry>();
    state.dir = Vec2::ZERO;
    touch.0 = None;
}

#[cfg(not(target_os = "android"))]
fn spawn_joystick() {}

#[cfg(not(target_os = "android"))]
fn despawn_joystick() {}

pub struct VirtualJoystickPlugin;

impl Plugin for VirtualJoystickPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "android")]
        {
            app.init_resource::<JoystickState>()
                .init_resource::<JoystickTouch>()
                .add_systems(OnEnter(AppState::InGame), spawn_joystick)
                .add_systems(
                    Update,
                    (
                        joystick_input
                            .after(HudWriteSet)
                            .before(super::camera::touch_pan_zoom)
                            .before(super::tokens::touch_interact),
                        update_thumb_position.after(joystick_input),
                        joystick_feedback.after(joystick_input),
                        joystick_move_token
                            .after(joystick_input)
                            .after(super::tokens::touch_interact)
                            .after(HudWriteSet),
                    )
                        .run_if(in_state(AppState::InGame)),
                )
                .add_systems(OnExit(AppState::InGame), despawn_joystick);
        }

        #[cfg(not(target_os = "android"))]
        {
            app.add_systems(OnEnter(AppState::InGame), spawn_joystick)
                .add_systems(OnExit(AppState::InGame), despawn_joystick);
        }
    }
}
