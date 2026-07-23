use bevy::prelude::*;

use crate::AppState;

#[cfg(target_os = "android")]
use super::camera::CamRig;
#[cfg(target_os = "android")]
use super::HudWriteSet;
#[cfg(target_os = "android")]
use super::ScreenInfo;
#[cfg(target_os = "android")]
use bevy::input::touch::TouchPhase;

#[allow(dead_code)]
const RING_BG: Color = Color::srgba(0.15, 0.14, 0.20, 0.45);
#[allow(dead_code)]
const THUMB_COLOR: Color = Color::srgba(0.83, 0.69, 0.22, 0.55);

#[derive(Resource, Default)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub struct JoystickState {
    pub left: Vec2,
    pub right: Vec2,
}

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct VirtualJoystickRoot;

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct LeftStick;

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct RightStick;

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct LeftThumb;

#[derive(Component)]
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
struct RightThumb;

#[cfg(target_os = "android")]
#[derive(Resource, Default)]
struct TouchAssignments {
    left_finger: Option<u64>,
    right_finger: Option<u64>,
}

#[cfg(target_os = "android")]
#[derive(Resource)]
struct StickGeometry {
    left_center: Vec2,
    right_center: Vec2,
    radius: f32,
    thumb_diameter: f32,
}

#[cfg(target_os = "android")]
fn spawn_joystick(
    mut commands: Commands,
    screen_info: Res<ScreenInfo>,
    existing: Query<Entity, With<VirtualJoystickRoot>>,
) {
    if !existing.is_empty() {
        return;
    }

    let diameter = 120.0 * screen_info.scale;
    let radius = diameter / 2.0;
    let margin = 40.0 * screen_info.scale;
    let thumb_diameter = diameter * 0.4;

    let left_x = margin + radius;
    let right_x = screen_info.width - margin - radius;
    let bottom_y = margin + radius;

    commands.insert_resource(StickGeometry {
        left_center: Vec2::new(left_x, bottom_y),
        right_center: Vec2::new(right_x, bottom_y),
        radius,
        thumb_diameter,
    });

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
            VirtualJoystickRoot,
        ))
        .with_children(|parent| {
            spawn_stick(
                parent,
                left_x,
                bottom_y,
                radius,
                diameter,
                thumb_diameter,
                LeftStick,
                LeftThumb,
            );
            spawn_stick(
                parent,
                right_x,
                bottom_y,
                radius,
                diameter,
                thumb_diameter,
                RightStick,
                RightThumb,
            );
        });
}

#[cfg(target_os = "android")]
fn spawn_stick(
    parent: &mut ChildSpawnerCommands,
    center_x: f32,
    center_y: f32,
    radius: f32,
    diameter: f32,
    thumb_diameter: f32,
    stick_marker: impl Component,
    thumb_marker: impl Component,
) {
    let thumb_offset = radius - thumb_diameter / 2.0;

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
            BorderColor::all(Color::srgba(0.83, 0.69, 0.22, 0.3)),
            stick_marker,
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
                thumb_marker,
            ));
        });
}

#[cfg(target_os = "android")]
fn joystick_input(
    mut touch_events: MessageReader<TouchInput>,
    mut state: ResMut<JoystickState>,
    mut assignments: ResMut<TouchAssignments>,
    geo: Res<StickGeometry>,
) {
    for event in touch_events.read() {
        let pos = event.position;
        match event.phase {
            TouchPhase::Started => {
                if pos.x < geo.left_center.x && assignments.left_finger.is_none() {
                    assignments.left_finger = Some(event.id);
                    clamp_to_stick(&mut state.left, pos, geo.left_center, geo.radius);
                } else if pos.x >= geo.left_center.x && assignments.right_finger.is_none() {
                    assignments.right_finger = Some(event.id);
                    clamp_to_stick(&mut state.right, pos, geo.right_center, geo.radius);
                }
            }
            TouchPhase::Moved => {
                if assignments.left_finger == Some(event.id) {
                    clamp_to_stick(&mut state.left, pos, geo.left_center, geo.radius);
                } else if assignments.right_finger == Some(event.id) {
                    clamp_to_stick(&mut state.right, pos, geo.right_center, geo.radius);
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if assignments.left_finger == Some(event.id) {
                    assignments.left_finger = None;
                    state.left = Vec2::ZERO;
                } else if assignments.right_finger == Some(event.id) {
                    assignments.right_finger = None;
                    state.right = Vec2::ZERO;
                }
            }
        }
    }
}

#[cfg(target_os = "android")]
fn clamp_to_stick(out: &mut Vec2, touch_pos: Vec2, center: Vec2, radius: f32) {
    let delta = touch_pos - center;
    let len = delta.length();
    let clamped = if len > radius {
        delta * (radius / len)
    } else {
        delta
    };
    *out = clamped / radius;
}

#[cfg(target_os = "android")]
fn update_thumb_positions(
    state: Res<JoystickState>,
    geo: Res<StickGeometry>,
    mut left_q: Query<&mut Node, With<LeftThumb>>,
    mut right_q: Query<&mut Node, (With<RightThumb>, Without<LeftThumb>)>,
) {
    let half = geo.thumb_diameter / 2.0;

    if let Ok(mut node) = left_q.single_mut() {
        let offset = state.left * geo.radius;
        node.left = Val::Px(geo.radius + offset.x - half);
        node.top = Val::Px(geo.radius + offset.y - half);
    }

    if let Ok(mut node) = right_q.single_mut() {
        let offset = state.right * geo.radius;
        node.left = Val::Px(geo.radius + offset.x - half);
        node.top = Val::Px(geo.radius + offset.y - half);
    }
}

#[cfg(target_os = "android")]
pub(crate) fn joystick_apply(
    state: Res<JoystickState>,
    mut rig: ResMut<CamRig>,
    screen_info: Res<ScreenInfo>,
    time: Res<Time>,
) {
    let pan_speed = 200.0 * screen_info.scale * time.delta_secs();
    let orbit_speed = 2.0 * time.delta_secs();

    let yaw_rot = Quat::from_rotation_y(rig.yaw);
    let right_vec = yaw_rot * Vec3::X;
    let fwd_vec = yaw_rot * Vec3::NEG_Z;

    rig.focus += right_vec * state.left.x * pan_speed;
    rig.focus += fwd_vec * state.left.y * pan_speed;

    rig.yaw -= state.right.x * orbit_speed;
    rig.pitch = (rig.pitch - state.right.y * orbit_speed).clamp(0.35, 1.45);
}

#[cfg(target_os = "android")]
fn despawn_joystick(mut commands: Commands, query: Query<Entity, With<VirtualJoystickRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<JoystickState>();
    commands.remove_resource::<TouchAssignments>();
    commands.remove_resource::<StickGeometry>();
}

#[cfg(not(target_os = "android"))]
fn spawn_joystick() {}

#[cfg(not(target_os = "android"))]
fn joystick_input() {}

#[cfg(not(target_os = "android"))]
pub(crate) fn joystick_apply() {}

#[cfg(not(target_os = "android"))]
fn despawn_joystick() {}

#[cfg(not(target_os = "android"))]
fn update_thumb_positions() {}

pub struct VirtualJoystickPlugin;

impl Plugin for VirtualJoystickPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "android")]
        {
            app.init_resource::<JoystickState>()
                .init_resource::<TouchAssignments>()
                .add_systems(OnEnter(AppState::InGame), spawn_joystick)
                .add_systems(
                    Update,
                    (
                        joystick_input.after(HudWriteSet),
                        update_thumb_positions.after(joystick_input),
                        joystick_apply
                            .after(joystick_input)
                            .after(super::camera::pan_zoom)
                            .after(super::camera::touch_pan_zoom)
                            .after(HudWriteSet),
                    )
                        .run_if(in_state(AppState::InGame)),
                )
                .add_systems(OnExit(AppState::InGame), despawn_joystick);
        }

        #[cfg(not(target_os = "android"))]
        {
            app.add_systems(OnEnter(AppState::InGame), spawn_joystick)
                .add_systems(
                    Update,
                    (joystick_input, update_thumb_positions, joystick_apply)
                        .run_if(in_state(AppState::InGame)),
                )
                .add_systems(OnExit(AppState::InGame), despawn_joystick);
        }
    }
}
