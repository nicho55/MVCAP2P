use std::sync::{Arc, Mutex};

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::game::ScreenInfo;
use crate::net::{Net, Roster, Session};
use crate::protocol::*;
use crate::room_discovery;
use crate::svg_assets::{tfont, GameAssets};
use crate::{AppState, CliArgs};

const GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
const PANEL: Color = Color::srgba(0.10, 0.09, 0.14, 0.96);
const FIELD_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const TEXT: Color = Color::srgb(0.92, 0.90, 0.95);
const MUTED: Color = Color::srgb(0.58, 0.55, 0.66);
const ROW_BG: Color = Color::srgba(0.20, 0.18, 0.26, 0.80);

/// Escala responsiva de um valor (similar ao `sz` do hud.rs).
fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

#[derive(Resource)]
pub struct RoomList {
    pub rooms: Vec<room_discovery::RoomEntry>,
    pub loading: bool,
    pub error: Option<String>,
    pending: Arc<Mutex<Option<Result<Vec<room_discovery::RoomEntry>, String>>>>,
    timer: Timer,
}

impl Default for RoomList {
    fn default() -> Self {
        Self {
            rooms: vec![],
            loading: false,
            error: None,
            pending: Arc::new(Mutex::new(None)),
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        }
    }
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbyForm>()
            .init_resource::<RoomList>()
            .add_systems(OnEnter(AppState::Lobby), setup_lobby)
            .add_systems(OnExit(AppState::Lobby), cleanup_lobby)
            .add_systems(
                Update,
                (
                    lobby_auto,
                    lobby_clicks,
                    lobby_typing,
                    lobby_reflect,
                    room_poll,
                    lobby_responsive,
                )
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
struct TestBtn;
#[derive(Component)]
struct JoinBtn;
#[derive(Component)]
struct RoomsPanel;
#[derive(Component)]
struct RoomsContainer;
#[derive(Component)]
struct RoomEntryBtn {
    code: String,
    signaling: String,
}
#[derive(Component)]
struct RoomEmptyLabel;

fn setup_lobby(mut commands: Commands, assets: Res<GameAssets>, si: Res<ScreenInfo>) {
    let p = sz(12.0, &si);
    let gap = sz(10.0, &si);
    let swatch = sz(40.0, &si);
    let input_h = sz(46.0, &si);
    let btn_h = sz(50.0, &si);
    let logo = sz(120.0, &si);
    commands
        .spawn((
            LobbyRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(gap),
                padding: UiRect::horizontal(Val::Px(p)),
                ..default()
            },
            Interaction::default(),
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(assets.logo.clone()),
                Node {
                    width: Val::Px(logo),
                    height: Val::Px(logo),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("TABLETOP P2P"),
                tfont(&assets, sz(36.0, &si)),
                TextColor(GOLD),
            ));
            root.spawn((
                Text::new("VTT tático peer-to-peer — WebRTC, sem servidor de jogo"),
                tfont(&assets, sz(14.0, &si)),
                TextColor(MUTED),
            ));
            // painel principal: formulário + salas lado a lado
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(sz(16.0, &si)),
                row_gap: Val::Px(sz(12.0, &si)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                max_width: Val::Px(sz(820.0, &si)),
                ..default()
            })
            .with_children(|row| {
                // coluna do formulário
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(gap),
                        padding: UiRect::all(Val::Px(sz(20.0, &si))),
                        flex_grow: 1.0,
                        min_width: Val::Px(sz(280.0, &si)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(Color::srgb(0.30, 0.26, 0.40)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("APELIDO"),
                        tfont(&assets, sz(13.0, &si)),
                        TextColor(MUTED),
                    ));
                    panel
                        .spawn((
                            Button,
                            NickField,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(input_h),
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(FIELD_BG),
                            BorderColor::all(GOLD),
                        ))
                        .with_children(|f| {
                            f.spawn((
                                NickText,
                                Text::new(""),
                                tfont(&assets, sz(18.0, &si)),
                                TextColor(TEXT),
                            ));
                        });
                    panel.spawn((
                        Text::new("SUA COR"),
                        tfont(&assets, sz(13.0, &si)),
                        TextColor(MUTED),
                    ));
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(sz(8.0, &si)),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        })
                        .with_children(|row| {
                            for i in 0..PALETTE.len() as u8 {
                                row.spawn((
                                    Button,
                                    Swatch(i),
                                    Node {
                                        width: Val::Px(swatch),
                                        height: Val::Px(swatch),
                                        border: UiRect::all(Val::Px(3.0)),
                                        ..default()
                                    },
                                    BackgroundColor(palette_color(i)),
                                    BorderColor::all(Color::NONE),
                                ));
                            }
                        });
                    panel
                        .spawn((
                            Button,
                            CreateBtn,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(btn_h),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::top(Val::Px(sz(4.0, &si))),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.55, 0.42, 0.12)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("CRIAR SALA (MESTRE)"),
                                tfont(&assets, sz(17.0, &si)),
                                TextColor(Color::srgb(0.98, 0.95, 0.88)),
                            ));
                        });
                    panel
                        .spawn((
                            Button,
                            TestBtn,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(btn_h * 0.8),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.22, 0.32, 0.52)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("SALA DE TESTE (GM + 4 TOKENS)"),
                                tfont(&assets, sz(14.0, &si)),
                                TextColor(Color::srgb(0.85, 0.90, 0.98)),
                            ));
                        });
                    panel.spawn((
                        Text::new("OU ENTRE COM UM CÓDIGO"),
                        tfont(&assets, sz(13.0, &si)),
                        TextColor(MUTED),
                    ));
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(sz(8.0, &si)),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                CodeField,
                                Node {
                                    flex_grow: 1.0,
                                    height: Val::Px(input_h),
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(Val::Px(12.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(FIELD_BG),
                                BorderColor::all(Color::srgb(0.30, 0.26, 0.40)),
                            ))
                            .with_children(|f| {
                                f.spawn((
                                    CodeText,
                                    Text::new(""),
                                    tfont(&assets, sz(18.0, &si)),
                                    TextColor(TEXT),
                                ));
                            });
                            row.spawn((
                                Button,
                                JoinBtn,
                                Node {
                                    width: Val::Px(sz(120.0, &si)),
                                    height: Val::Px(input_h),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.32, 0.52)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("ENTRAR"),
                                    tfont(&assets, sz(16.0, &si)),
                                    TextColor(TEXT),
                                ));
                            });
                        });
                    panel.spawn((
                        StatusText,
                        Text::new(""),
                        tfont(&assets, sz(14.0, &si)),
                        TextColor(Color::srgb(0.92, 0.45, 0.45)),
                    ));
                });
                // coluna da lista de salas
                row.spawn((
                    RoomsPanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 0.5,
                        min_width: Val::Px(sz(220.0, &si)),
                        padding: UiRect::all(Val::Px(sz(14.0, &si))),
                        row_gap: Val::Px(sz(8.0, &si)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(Color::srgb(0.30, 0.26, 0.40)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("SALAS ABERTAS"),
                        tfont(&assets, sz(14.0, &si)),
                        TextColor(GOLD),
                    ));
                    panel.spawn((
                        RoomsContainer,
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(sz(6.0, &si)),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                });
            });
            root.spawn((
                Text::new("Requer o servidor de sinalização: cargo run -p signaling  •  Tab alterna campos  •  F12 = screenshot"),
                tfont(&assets, sz(12.0, &si)),
                TextColor(Color::srgb(0.45, 0.43, 0.52)),
            ));
        });
}

fn cleanup_lobby(mut commands: Commands, q: Query<Entity, With<LobbyRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// Reconstrói o lobby quando a escala muda (janela redimensionada / rotação).
fn lobby_responsive(
    si: Res<ScreenInfo>,
    q_root: Query<Entity, With<LobbyRoot>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    if !si.is_changed() {
        return;
    }
    for e in &q_root {
        commands.entity(e).despawn();
    }
    setup_lobby(commands, assets, si);
}

fn start_session(
    gm: bool,
    join_code: Option<String>,
    signaling_override: Option<&str>,
    is_test_room: bool,
    form: &mut LobbyForm,
    args: &CliArgs,
    net: &mut Net,
    roster: &mut Roster,
    commands: &mut Commands,
    next: &mut NextState<AppState>,
) {
    // Código informado (join/CLI): valida estrito e, se vier torto, saneia.
    let normalize = |s: &str| RoomCode::parse(s).unwrap_or_else(|| RoomCode::sanitized(s));
    let code: RoomCode = match join_code {
        Some(c) => normalize(&c),
        None => args
            .code
            .as_deref()
            .map(normalize)
            .unwrap_or_else(RoomCode::generate),
    };
    let host = signaling_override
        .map(|s| s.to_string())
        .or_else(|| args.signaling.clone())
        .or_else(|| std::env::var("TABLETOP_SIGNALING").ok())
        .unwrap_or_else(|| "127.0.0.1:3536".into());
    let url = format!("ws://{host}/tabletop_{code}");
    let nick = {
        let t = form.nick.trim();
        if t.is_empty() {
            "Jogador".to_string()
        } else {
            t.to_string()
        }
    };
    let me = PlayerMeta {
        uuid: rand::random(),
        nick: nick.clone(),
        color: ColorIdx::new(form.color),
        is_gm: gm,
    };
    net.connect(&url);
    roster.list.clear();
    roster.upsert(me.clone(), None);
    info!(
        "sessão iniciada — sala {code} ({})",
        if gm { "mestre" } else { "jogador" }
    );
    // publica sala no Supabase com IP da LAN (não 127.0.0.1)
    if gm {
        let code = code.clone();
        let nick = nick.clone();
        let publish_host = if host.starts_with("127.0.0.1") || host.starts_with("localhost") {
            detect_lan_ip().unwrap_or(host.clone())
        } else {
            host.clone()
        };
        let ph = publish_host.clone();
        std::thread::spawn(move || {
            let _ = room_discovery::create_room(code.as_str(), &nick, &ph);
        });
        info!("sala publicada no Supabase com host {publish_host}");
    }
    commands.insert_resource(Session {
        me,
        code,
        is_test_room,
    });
    next.set(AppState::InGame);
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
        start_session(
            true,
            None,
            None,
            false,
            &mut form,
            &args,
            &mut net,
            &mut roster,
            &mut commands,
            &mut next,
        );
    } else if let Some(code) = args.join.clone() {
        start_session(
            false,
            Some(code.to_uppercase()),
            None,
            false,
            &mut form,
            &args,
            &mut net,
            &mut roster,
            &mut commands,
            &mut next,
        );
    }
}

fn lobby_clicks(
    mut form: ResMut<LobbyForm>,
    q_nick: Query<&Interaction, (Changed<Interaction>, With<NickField>)>,
    q_code: Query<&Interaction, (Changed<Interaction>, With<CodeField>)>,
    q_swatch: Query<(&Interaction, &Swatch), Changed<Interaction>>,
    q_create: Query<&Interaction, (Changed<Interaction>, With<CreateBtn>)>,
    q_test: Query<&Interaction, (Changed<Interaction>, With<TestBtn>)>,
    q_join: Query<&Interaction, (Changed<Interaction>, With<JoinBtn>)>,
    q_room_entry: Query<(&Interaction, &RoomEntryBtn), Changed<Interaction>>,
    mut args: ResMut<CliArgs>,
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
            start_session(
                true,
                None,
                None,
                false,
                &mut form,
                &args,
                &mut net,
                &mut roster,
                &mut commands,
                &mut next,
            );
        }
    }
    for i in &q_test {
        if *i == Interaction::Pressed {
            args.demo = true;
            start_session(
                true,
                None,
                None,
                true,
                &mut form,
                &args,
                &mut net,
                &mut roster,
                &mut commands,
                &mut next,
            );
        }
    }
    for i in &q_join {
        if *i == Interaction::Pressed {
            let code = form.code.trim().to_uppercase();
            if code.len() < 4 {
                form.status = "Código inválido (mínimo 4 caracteres)".into();
            } else {
                start_session(
                    false,
                    Some(code),
                    None,
                    false,
                    &mut form,
                    &args,
                    &mut net,
                    &mut roster,
                    &mut commands,
                    &mut next,
                );
            }
        }
    }
    for (i, btn) in &q_room_entry {
        if *i == Interaction::Pressed {
            form.code = btn.code.clone();
            start_session(
                false,
                Some(btn.code.clone()),
                Some(&btn.signaling),
                false,
                &mut form,
                &args,
                &mut net,
                &mut roster,
                &mut commands,
                &mut next,
            );
        }
    }
}

fn lobby_typing(mut form: ResMut<LobbyForm>, mut keys: MessageReader<KeyboardInput>) {
    for ev in keys.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        if matches!(ev.logical_key, Key::Tab) {
            form.focus = if form.focus == Focus::Nick {
                Focus::Code
            } else {
                Focus::Nick
            };
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
        *b = BorderColor::all(if form.focus == Focus::Nick { GOLD } else { dim });
    }
    for mut b in &mut q_code_b {
        *b = BorderColor::all(if form.focus == Focus::Code { GOLD } else { dim });
    }
    for (s, mut b) in &mut q_swatches {
        *b = BorderColor::all(if s.0 == form.color {
            Color::WHITE
        } else {
            Color::NONE
        });
    }
}

fn room_poll(
    mut list: ResMut<RoomList>,
    q_container: Query<Entity, With<RoomsContainer>>,
    q_entries: Query<Entity, With<RoomEntryBtn>>,
    q_empty: Query<Entity, With<RoomEmptyLabel>>,
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    si: Res<ScreenInfo>,
) {
    list.timer.tick(time.delta());
    let result = { list.pending.lock().ok().and_then(|mut p| p.take()) };
    if let Some(result) = result {
        list.loading = false;
        match result {
            Ok(rooms) => {
                list.rooms = rooms;
                list.error = None;
            }
            Err(e) => {
                list.error = Some(e);
            }
        }
    }
    if list.timer.just_finished() && !list.loading {
        list.loading = true;
        let pending = list.pending.clone();
        std::thread::spawn(move || {
            let result = room_discovery::list_rooms();
            if let Ok(mut p) = pending.lock() {
                *p = Some(result);
            }
        });
    }
    let Ok(container) = q_container.single() else {
        return;
    };
    for e in q_entries.iter().chain(q_empty.iter()) {
        commands.entity(e).despawn();
    }
    commands.entity(container).with_children(|c| {
        if list.rooms.is_empty() {
            let msg = if list.loading {
                "Carregando..."
            } else if list.error.is_some() {
                "Erro ao carregar"
            } else {
                "Nenhuma sala aberta"
            };
            c.spawn((
                RoomEmptyLabel,
                Text::new(msg),
                tfont(&assets, sz(13.0, &si)),
                TextColor(MUTED),
            ));
        } else {
            for entry in &list.rooms {
                let age = age_str(&entry.created_at);
                let label = format!("{}  •  {} ({})", entry.gm_name, entry.code, age);
                c.spawn((
                    Button,
                    RoomEntryBtn {
                        code: entry.code.clone(),
                        signaling: entry.room_url.clone(),
                    },
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(ROW_BG),
                    BorderColor::all(Color::srgb(0.30, 0.26, 0.40)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        tfont(&assets, sz(12.0, &si)),
                        TextColor(TEXT),
                    ));
                });
            }
        }
    });
}

fn age_str(created_at: &str) -> String {
    if created_at.len() >= 19 {
        let t = &created_at[11..19];
        format!("{} UTC", t)
    } else {
        String::new()
    }
}

/// Detecta o IP da LAN conectando um UDP socket a um endereço externo
/// (sem enviar dados) e lendo o endereço local.
fn detect_lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:53").ok()?;
    let addr = socket.local_addr().ok()?;
    let ip = addr.ip();
    if ip.is_loopback() {
        None
    } else {
        Some(format!("{ip}:3536"))
    }
}
