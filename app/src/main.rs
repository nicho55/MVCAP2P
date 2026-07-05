#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod game;
mod lobby;
mod net;
mod protocol;
mod svg_assets;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Estado inicial neutro: sem OnEnter dependente de assets.
    /// A transição inicial do bevy_state roda antes do PreStartup (onde os SVGs
    /// são rasterizados), então o estado default NÃO pode depender de GameAssets.
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

fn main() {
    let args = parse_args();
    let mut title = String::from("Tabletop P2P");
    if args.gm {
        title.push_str(" — GM");
    } else if args.join.is_some() {
        title.push_str(" — Jogador");
    }
    if let Some(n) = &args.nick {
        title.push_str(&format!(" ({n})"));
    }
    App::new()
        .insert_resource(args)
        .insert_resource(ClearColor(Color::srgb(0.075, 0.065, 0.10)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title,
                        resolution: (1366.0, 840.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                // WebRTC em LAN gera muitos WARN inofensivos (IPv6 link-local sem escopo,
                // STUN inalcançável). Silencia esse ruído sem esconder erros nossos.
                .set(bevy::log::LogPlugin {
                    filter: "info,wgpu=error,naga=warn,webrtc_ice=error,webrtc=error,webrtc_mdns=error".into(),
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .init_state::<AppState>()
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

/// Roda no Startup (após PreStartup rasterizar os SVGs) e libera o lobby.
fn boot_to_lobby(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Lobby);
}

fn screenshot_hotkey(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands, mut n: Local<u32>) {
    if keys.just_pressed(KeyCode::F12) {
        *n += 1;
        let path = format!("screenshot_{}.png", *n);
        info!("screenshot -> {path}");
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
}

fn auto_shot_exit(
    time: Res<Time>,
    args: Res<CliArgs>,
    mut commands: Commands,
    mut done: Local<bool>,
    mut exit: EventWriter<AppExit>,
) {
    if let Some(path) = &args.shot {
        if !*done && time.elapsed_secs() > args.shot_at {
            *done = true;
            info!("screenshot automático -> {path}");
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
        }
    }
    if let Some(at) = args.exit_at {
        if time.elapsed_secs() > at {
            exit.write(AppExit::Success);
        }
    }
}
