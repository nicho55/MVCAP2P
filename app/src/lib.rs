#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod game;
mod lobby;
mod net;
mod protocol;
mod room_discovery;
mod svg_assets;
pub mod transcode;

use bevy::prelude::*;
#[cfg(target_os = "android")]
use bevy::render::settings::{Backends, RenderCreation, WgpuLimits, WgpuSettings};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
#[cfg(target_os = "android")]
use bevy::render::RenderPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Boot,
    Lobby,
    InGame,
}

#[derive(Resource, Debug, Clone)]
pub struct CliArgs {
    pub gm: bool,
    pub join: Option<String>,
    pub code: Option<String>,
    pub nick: Option<String>,
    pub color: Option<u8>,
    pub map: Option<String>,
    pub demo: bool,
    pub signaling: Option<String>,
    pub shot: Option<String>,
    pub shot_at: f32,
    pub exit_at: Option<f32>,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            gm: false,
            join: None,
            code: None,
            nick: None,
            color: None,
            map: None,
            demo: false,
            signaling: None,
            shot: None,
            shot_at: 6.0,
            exit_at: None,
        }
    }
}

#[cfg(not(target_os = "android"))]
fn parse_args() -> CliArgs {
    let mut a = CliArgs::default();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--gm" => a.gm = true,
            "--demo" => a.demo = true,
            "--join" => a.join = it.next(),
            "--code" => a.code = it.next(),
            "--nick" => a.nick = it.next(),
            "--color" => a.color = it.next().and_then(|v| v.parse().ok()),
            "--map" => a.map = it.next(),
            "--signaling" => a.signaling = it.next(),
            "--shot" => a.shot = it.next(),
            "--shot-at" => a.shot_at = it.next().and_then(|v| v.parse().ok()).unwrap_or(6.0),
            "--exit-at" => a.exit_at = it.next().and_then(|v| v.parse().ok()),
            other => eprintln!("argumento desconhecido: {other}"),
        }
    }
    a
}

pub fn run_game() {
    #[cfg(not(target_os = "android"))]
    let args = parse_args();
    #[cfg(target_os = "android")]
    let args = CliArgs::default();

    #[cfg(not(target_os = "android"))]
    let mut title = String::from("Tabletop P2P");
    #[cfg(not(target_os = "android"))]
    {
        if args.gm {
            title.push_str(" — GM");
        } else if args.join.is_some() {
            title.push_str(" — Jogador");
        }
        if let Some(n) = &args.nick {
            title.push_str(&format!(" ({n})"));
        }
    }

    let mut app = App::new();
    app.insert_resource(args)
        .insert_resource(ClearColor(Color::srgb(0.075, 0.065, 0.10)));

    #[cfg(not(target_os = "android"))]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title,
                        resolution: bevy::window::WindowResolution::new(1366, 840),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    filter:
                        "info,wgpu=error,naga=warn,webrtc_ice=error,webrtc=error,webrtc_mdns=error"
                            .into(),
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        );
    }

    #[cfg(target_os = "android")]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    filter: "info,wgpu=error,naga=warn".into(),
                    level: bevy::log::Level::INFO,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN | Backends::GL),
                        limits: WgpuLimits::downlevel_webgl2_defaults(),
                        ..default()
                    }),
                    ..default()
                }),
        );
    }

    app.init_state::<AppState>()
        .add_plugins((
            svg_assets::SvgAssetsPlugin,
            net::NetPlugin,
            lobby::LobbyPlugin,
            game::GamePlugin,
        ))
        .add_systems(Startup, boot_to_lobby)
        .add_systems(Update, (screenshot_hotkey, auto_shot_exit))
        .run();
}

#[cfg(target_os = "android")]
#[bevy_main]
fn main() {
    run_game();
}

fn boot_to_lobby(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Lobby);
}

fn screenshot_hotkey(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands, mut n: Local<u32>) {
    if keys.just_pressed(KeyCode::F12) {
        *n += 1;
        let path = format!("screenshot_{}.png", *n);
        info!("screenshot -> {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn auto_shot_exit(
    time: Res<Time>,
    args: Res<CliArgs>,
    mut commands: Commands,
    mut done: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    if let Some(path) = &args.shot {
        if !*done && time.elapsed_secs() > args.shot_at {
            *done = true;
            info!("screenshot automático -> {path}");
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.clone()));
        }
    }
    if let Some(at) = args.exit_at {
        if time.elapsed_secs() > at {
            exit.write(AppExit::Success);
        }
    }
}
