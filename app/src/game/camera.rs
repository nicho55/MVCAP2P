use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::IsDefaultUiCamera;

#[cfg(target_os = "android")]
use bevy::input::touch::TouchPhase;
#[cfg(target_os = "android")]
use bevy::platform::collections::HashMap;

#[cfg(target_os = "android")]
use super::tokens::TouchDrag;
use super::UiHovered;

#[derive(Component)]
pub struct MainCamera;

/// Câmera orbital estilo RPG tático: foco no chão, yaw/pitch/distância.
#[derive(Resource)]
pub struct CamRig {
    pub focus: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
}

impl Default for CamRig {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.95,
            dist: 1400.0,
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            far: 30000.0,
            ..default()
        }),
        Transform::default(),
        // Componente MSAA presente para ser ajustado em runtime por
        // GraphicsSettings (apply_graphics define o nível real no 1º frame).
        Msaa::Off,
        // Bevy 0.18: AmbientLight virou componente (por câmera), não Resource.
        AmbientLight {
            color: Color::srgb(0.85, 0.88, 1.0),
            brightness: 350.0,
            ..default()
        },
        MainCamera,
    ));

    // Câmera de UI dedicada: renderiza a interface num passe separado, por cima
    // do 3D (order=1, sem limpar o buffer). No Android a sub-passe de UI da
    // câmera 3D falhava de forma intermitente (Bevy #14710), sumindo o HUD sobre
    // o mapa. Um passe próprio garante a composição da UI sobre a cena. (#43)
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Msaa::Off,
        IsDefaultUiCamera,
    ));
}

pub fn apply_rig(rig: Res<CamRig>, mut q: Query<&mut Transform, With<MainCamera>>) {
    let Ok(mut t) = q.single_mut() else { return };
    let rot = Quat::from_euler(EulerRot::YXZ, rig.yaw, -rig.pitch, 0.0);
    t.translation = rig.focus + rot * Vec3::new(0.0, 0.0, rig.dist);
    t.look_at(rig.focus, Vec3::Y);
}

pub fn pan_zoom(
    mut wheel: MessageReader<MouseWheel>,
    mut motion: MessageReader<MouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut rig: ResMut<CamRig>,
    ui: Res<UiHovered>,
) {
    let mut delta = Vec2::ZERO;
    for m in motion.read() {
        delta += m.delta;
    }

    let yaw_rot = Quat::from_rotation_y(rig.yaw);
    let right = yaw_rot * Vec3::X;
    let fwd = yaw_rot * Vec3::NEG_Z;

    // pan: botão direito arrasta o chão ("grab")
    if buttons.pressed(MouseButton::Right) && delta != Vec2::ZERO {
        let k = rig.dist * 0.0016;
        rig.focus -= right * delta.x * k;
        rig.focus += fwd * delta.y * k;
    }

    // rotação: botão do meio (yaw horizontal, pitch vertical)
    if buttons.pressed(MouseButton::Middle) && delta != Vec2::ZERO {
        rig.yaw -= delta.x * 0.006;
        rig.pitch = (rig.pitch + delta.y * 0.004).clamp(0.35, 1.45);
    }

    // teclado: WASD/setas pan, Q/E yaw
    let dt = time.delta_secs();
    let spd = rig.dist * 1.1 * dt;
    let mut kmove = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        kmove += fwd;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        kmove -= fwd;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        kmove += right;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        kmove -= right;
    }
    if kmove != Vec3::ZERO {
        rig.focus += kmove.normalize() * spd;
    }
    if keys.pressed(KeyCode::KeyQ) {
        rig.yaw += 1.8 * dt;
    }
    if keys.pressed(KeyCode::KeyE) {
        rig.yaw -= 1.8 * dt;
    }

    // zoom: scroll aproxima/afasta do foco
    let mut scroll = 0.0f32;
    for w in wheel.read() {
        scroll += match w.unit {
            MouseScrollUnit::Line => w.y,
            MouseScrollUnit::Pixel => w.y / 100.0,
        };
    }
    if scroll.abs() > 0.001 && !ui.0 {
        rig.dist = (rig.dist * 0.88f32.powf(scroll)).clamp(180.0, 8000.0);
    }
}

/// Raio do cursor no mundo.
pub fn cursor_ray(win: &Window, cam: &Camera, cam_gt: &GlobalTransform) -> Option<Ray3d> {
    let c = win.cursor_position()?;
    cam.viewport_to_world(cam_gt, c).ok()
}

/// Interseção do raio com o plano do chão (y = altura dada).
pub fn ray_ground(ray: Ray3d, y: f32) -> Option<Vec3> {
    let d = ray.intersect_plane(Vec3::new(0.0, y, 0.0), InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(d))
}

/// Ponto do chão sob o cursor, como Vec2 = (x, z) — casa com a matemática do grid.
pub fn cursor_ground(win: &Window, cam: &Camera, cam_gt: &GlobalTransform) -> Option<Vec2> {
    let p = ray_ground(cursor_ray(win, cam, cam_gt)?, 0.0)?;
    Some(Vec2::new(p.x, p.z))
}

/// Distância mínima entre um raio e um ponto (para picking de tokens 3D).
pub fn ray_point_dist(ray: &Ray3d, p: Vec3) -> f32 {
    let v = p - ray.origin;
    let d = *ray.direction;
    (v - d * v.dot(d)).length()
}

// ─── Touch camera controls (Android) ────────────────────────────────────────

#[cfg(target_os = "android")]
#[derive(Resource, Default)]
pub struct TouchState {
    pub fingers: HashMap<u64, Vec2>,
    pub last_pinch: Option<f32>,
}

/// Touch-based camera pan/zoom/orbit (Android).
/// - 1 finger drag → pan (right-click equivalent)
/// - 2 finger drag → orbit (middle-click equivalent)
/// - pinch → zoom (scroll equivalent)
#[cfg(target_os = "android")]
pub fn touch_pan_zoom(
    mut touch_ev: MessageReader<TouchInput>,
    mut state: ResMut<TouchState>,
    mut rig: ResMut<CamRig>,
    ui: Res<UiHovered>,
    drag: Res<TouchDrag>,
) {
    if drag.token_id.is_some() {
        return;
    }
    for t in touch_ev.read() {
        if ui.0 && matches!(t.phase, TouchPhase::Started) {
            continue;
        }
        match t.phase {
            TouchPhase::Started => {
                state.fingers.insert(t.id, t.position);
                state.last_pinch = None;
            }
            TouchPhase::Moved => {
                if let Some(&old) = state.fingers.get(&t.id) {
                    state.fingers.insert(t.id, t.position);

                    let yaw_rot = Quat::from_rotation_y(rig.yaw);
                    let right = yaw_rot * Vec3::X;
                    let fwd = yaw_rot * Vec3::NEG_Z;
                    let k = rig.dist * 0.0016;

                    let count = state.fingers.len();
                    match count {
                        1 => {
                            let delta = t.position - old;
                            rig.focus -= right * delta.x * k;
                            rig.focus += fwd * delta.y * k;
                        }
                        2 => {
                            let mut iter = state.fingers.iter();
                            let (_, &p1) = iter.next().unwrap();
                            let (_, &p2) = iter.next().unwrap();
                            let center = (p1 + p2) / 2.0;
                            let dist = p1.distance(p2);

                            // orbit: delta do centro
                            let dcenter = center
                                - (old + state.fingers.get(&t.id).copied().unwrap_or(t.position))
                                    / 2.0;
                            rig.yaw -= dcenter.x * 0.006;
                            rig.pitch = (rig.pitch + dcenter.y * 0.004).clamp(0.35, 1.45);

                            // pinch zoom
                            if let Some(last) = state.last_pinch {
                                let ratio = dist / last;
                                if (ratio - 1.0).abs() > 0.01 {
                                    rig.dist = (rig.dist / ratio).clamp(180.0, 8000.0);
                                }
                            }
                            state.last_pinch = Some(dist);
                        }
                        _ => {}
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                state.fingers.remove(&t.id);
                if state.fingers.len() < 2 {
                    state.last_pinch = None;
                }
            }
        }
    }
}
