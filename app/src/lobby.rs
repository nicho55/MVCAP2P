use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::net::{Net, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::{tfont, GameAssets};
use crate::{AppState, CliArgs};

const GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
const PANEL: Color = Color::srgba(0.10, 0.09, 0.14, 0.96);
const FIELD_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const TEXT: Color = Color::srgb(0.92, 0.90, 0.95);
const MUTED: Color = Color::srgb(0.58, 0.55, 0.66);

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbyForm>()
            .add_systems(OnEnter(AppState::Lobby), setup_lobby)
            .add_systems(OnExit(AppState::Lobby), cleanup_lobby)
            .add_systems(
                Update,
                (lobby_auto, lobby_clicks, lobby_typing, lobby_reflect)
                    .run_if(in_state(AppState::Lobby)),
            );
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
enum Focus {
    #[default]
    Nick,
    Code,
}

#[derive(Resource)]
struct LobbyForm {
    nick: String,
    code: String,
    color: u8,
    focus: Focus,
    status: String,
}

impl Default for LobbyForm {
    fn default() -> Self {
        Self {
            nick: format!("Jogador{}", rand::random::<u16>() % 90 + 10),
            code: String::new(),
            color: rand::random::<u8>() % 8,
            focus: Focus::Nick,
            status: String::new(),
        }
    }
}

#[derive(Component)]
struct LobbyRoot;
#[derive(Component)]
struct NickField;
#[derive(Component)]
struct CodeField;
#[derive(Component)]
struct NickText;
#[derive(Component)]
struct CodeText;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct Swatch(u8);
#[derive(Component)]
struct CreateBtn;
#[derive(Component)]
struct JoinBtn;

fn setup_lobby(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            LobbyRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            Interaction::default(),
        ))
        .with_children(|p| {
            p.spawn((
                ImageNode::new(assets.logo.clone()),
                Node { width: Val::Px(140.0), height: Val::Px(140.0), ..default() },
            ));
            p.spawn((Text::new("TABLETOP P2P"), tfont(&assets, 42.0), TextColor(GOLD)));
            p.spawn((
                Text::new("VTT tático peer-to-peer — WebRTC, sem servidor de jogo"),
                tfont(&assets, 15.0),
                TextColor(MUTED),
            ));
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    width: Val::Px(440.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor(Color::srgb(0.30, 0.26, 0.40)),
                BorderRadius::all(Val::Px(12.0)),
            ))
            .with_children(|panel| {
                panel.spawn((Text::new("APELIDO"), tfont(&assets, 13.0), TextColor(MUTED)));
                panel
                    .spawn((
                        Button,
                        NickField,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(42.0),
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(FIELD_BG),
                        BorderColor(GOLD),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|f| {
                        f.spawn((NickText, Text::new(""), tfont(&assets, 18.0), TextColor(TEXT)));
                    });
                panel.spawn((Text::new("SUA COR"), tfont(&assets, 13.0), TextColor(MUTED)));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        for i in 0..PALETTE.len() as u8 {
                            row.spawn((
                                Button,
                                Swatch(i),
                                Node {
                                    width: Val::Px(36.0),
                                    height: Val::Px(36.0),
                                    border: UiRect::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(palette_color(i)),
                                BorderColor(Color::NONE),
                                BorderRadius::all(Val::Px(18.0)),
                            ));
                        }
                    });
                panel
                    .spawn((
                        Button,
                        CreateBtn,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(46.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.55, 0.42, 0.12)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("CRIAR SALA (MESTRE)"), tfont(&assets, 18.0), TextColor(Color::srgb(0.98, 0.95, 0.88))));
                    });
                panel.spawn((Text::new("OU ENTRE COM UM CÓDIGO"), tfont(&assets, 13.0), TextColor(MUTED)));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Button,
                            CodeField,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(42.0),
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(FIELD_BG),
                            BorderColor(Color::srgb(0.30, 0.26, 0.40)),
                            BorderRadius::all(Val::Px(6.0)),
                        ))
                        .with_children(|f| {
                            f.spawn((CodeText, Text::new(""), tfont(&assets, 18.0), TextColor(TEXT)));
                        });
                        row.spawn((
                            Button,
                            JoinBtn,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(42.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.22, 0.32, 0.52)),
                            BorderRadius::all(Val::Px(8.0)),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::new("ENTRAR"), tfont(&assets, 17.0), TextColor(TEXT)));
                        });
                    });
                panel.spawn((StatusText, Text::new(""), tfont(&assets, 14.0), TextColor(Color::srgb(0.92, 0.45, 0.45))));
            });
            p.spawn((
                Text::new("Requer o servidor de sinalização: cargo run -p signaling  •  Tab alterna campos  •  F12 = screenshot"),
                tfont(&assets, 13.0),
                TextColor(Color::srgb(0.45, 0.43, 0.52)),
            ));
        });
}

fn cleanup_lobby(mut commands: Commands, q: Query<Entity, With<LobbyRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn lobby_auto(
    mut ran: Local<bool>,
    args: Res<CliArgs>,
    mut form: ResMut<LobbyForm>,
    mut net: ResMut<Net>,
    mut roster: ResMut<Roster>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    if *ran {
        return;
    }
    *ran = true;
    if let Some(n) = &args.nick {
        form.nick = n.clone();
    }
    if let Some(c) = args.color {
        form.color = c % 8;
    }
    if args.gm {
        start_session(true, None, &mut form, &args, &mut net, &mut roster, &mut commands, &mut next);
    } else if let Some(code) = args.join.clone() {
        start_session(false, Some(code.to_uppercase()), &mut form, &args, &mut net, &mut roster, &mut commands, &mut next);
    }
}

fn lobby_clicks(
    mut form: ResMut<LobbyForm>,
    q_nick: Query<&Interaction, (Changed<Interaction>, With<NickField>)>,
    q_code: Query<&Interaction, (Changed<Interaction>, With<CodeField>)>,
    q_swatch: Query<(&Interaction, &Swatch), Changed<Interaction>>,
    q_create: Query<&Interaction, (Changed<Interaction>, With<CreateBtn>)>,
    q_join: Query<&Interaction, (Changed<Interaction>, With<JoinBtn>)>,
    args: Res<CliArgs>,
    mut net: ResMut<Net>,
    mut roster: ResMut<Roster>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    for i in &q_nick {
        if *i == Interaction::Pressed {
            form.focus = Focus::Nick;
        }
    }
    for i in &q_code {
        if *i == Interaction::Pressed {
            form.focus = Focus::Code;
        }
    }
    for (i, s) in &q_swatch {
        if *i == Interaction::Pressed {
            form.color = s.0;
        }
    }
    for i in &q_create {
        if *i == Interaction::Pressed {
            start_session(true, None, &mut form, &args, &mut net, &mut roster, &mut commands, &mut next);
        }
    }
    for i in &q_join {
        if *i == Interaction::Pressed {
            let code = form.code.trim().to_uppercase();
            if code.len() < 4 {
                form.status = "Código inválido (mínimo 4 caracteres)".into();
            } else {
                start_session(false, Some(code), &mut form, &args, &mut net, &mut roster, &mut commands, &mut next);
            }
        }
    }
}

fn lobby_typing(mut form: ResMut<LobbyForm>, mut keys: EventReader<KeyboardInput>) {
    for ev in keys.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        if matches!(ev.logical_key, Key::Tab) {
            form.focus = if form.focus == Focus::Nick { Focus::Code } else { Focus::Nick };
            continue;
        }
        let code_mode = form.focus == Focus::Code;
        let max = if code_mode { 6 } else { 16 };
        let buf = match form.focus {
            Focus::Nick => &mut form.nick,
            Focus::Code => &mut form.code,
        };
        match &ev.logical_key {
            Key::Character(s) => {
                for ch in s.chars() {
                    if buf.chars().count() >= max {
                        break;
                    }
                    if code_mode {
                        if ch.is_ascii_alphanumeric() {
                            buf.push(ch.to_ascii_uppercase());
                        }
                    } else if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                        buf.push(ch);
                    }
                }
            }
            Key::Space => {
                if !code_mode && buf.chars().count() < max {
                    buf.push(' ');
                }
            }
            Key::Backspace => {
                buf.pop();
            }
            _ => {}
        }
    }
}

fn lobby_reflect(
    form: Res<LobbyForm>,
    mut q_nick_text: Query<&mut Text, (With<NickText>, Without<CodeText>, Without<StatusText>)>,
    mut q_code_text: Query<&mut Text, (With<CodeText>, Without<NickText>, Without<StatusText>)>,
    mut q_status: Query<&mut Text, (With<StatusText>, Without<NickText>, Without<CodeText>)>,
    mut q_nick_b: Query<&mut BorderColor, (With<NickField>, Without<CodeField>, Without<Swatch>)>,
    mut q_code_b: Query<&mut BorderColor, (With<CodeField>, Without<NickField>, Without<Swatch>)>,
    mut q_swatches: Query<(&Swatch, &mut BorderColor), (Without<NickField>, Without<CodeField>)>,
) {
    if !form.is_changed() {
        return;
    }
    let cursor_n = if form.focus == Focus::Nick { "_" } else { "" };
    let cursor_c = if form.focus == Focus::Code { "_" } else { "" };
    for mut t in &mut q_nick_text {
        t.0 = format!("{}{}", form.nick, cursor_n);
    }
    for mut t in &mut q_code_text {
        let display = if form.code.is_empty() && form.focus != Focus::Code {
            "CÓDIGO".to_string()
        } else {
            format!("{}{}", form.code, cursor_c)
        };
        t.0 = display;
    }
    for mut t in &mut q_status {
        t.0 = form.status.clone();
    }
    let dim = Color::srgb(0.30, 0.26, 0.40);
    for mut b in &mut q_nick_b {
        b.0 = if form.focus == Focus::Nick { GOLD } else { dim };
    }
    for mut b in &mut q_code_b {
        b.0 = if form.focus == Focus::Code { GOLD } else { dim };
    }
    for (s, mut b) in &mut q_swatches {
        b.0 = if s.0 == form.color { Color::WHITE } else { Color::NONE };
    }
}

fn random_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..5).map(|_| CODE_ALPHABET[rng.gen_range(0..CODE_ALPHABET.len())] as char).collect()
}

fn start_session(
    gm: bool,
    join_code: Option<String>,
    form: &mut LobbyForm,
    args: &CliArgs,
    net: &mut Net,
    roster: &mut Roster,
    commands: &mut Commands,
    next: &mut NextState<AppState>,
) {
    let code = match join_code {
        Some(c) => c,
        None => args.code.clone().map(|c| c.to_uppercase()).unwrap_or_else(random_code),
    };
    let host = args
        .signaling
        .clone()
        .or_else(|| std::env::var("TABLETOP_SIGNALING").ok())
        .unwrap_or_else(|| "127.0.0.1:3536".into());
    let url = format!("ws://{host}/tabletop_{code}");
    let nick = {
        let t = form.nick.trim();
        if t.is_empty() { "Jogador".to_string() } else { t.to_string() }
    };
    let me = PlayerMeta { uuid: rand::random(), nick, color: form.color, is_gm: gm };
    net.connect(&url);
    roster.list.clear();
    roster.upsert(me.clone(), None);
    info!("sessão iniciada — sala {code} ({})", if gm { "mestre" } else { "jogador" });
    commands.insert_resource(Session { me, code });
    next.set(AppState::InGame);
}
