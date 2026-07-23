use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use super::lowpoly::Ctx3d;
use super::map::DropMode;
use super::tokens::{
    delete_selected_entity, set_token_owner, OwnerRing, Selection as TokenSelection, Token,
};
use super::ActiveTool;
use super::ScreenInfo;
use crate::net::{Net, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::{tfont, GameAssets};
use crate::AppState;
use crate::DeviceProfile;

use super::grid::GridRes;

const GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
const TEXT: Color = Color::srgb(0.92, 0.90, 0.95);
const PANEL: Color = Color::srgba(0.10, 0.09, 0.14, 0.94);
const PANEL_SOFT: Color = Color::srgba(0.10, 0.09, 0.14, 0.75);
const BTN_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const BTN_BORDER: Color = Color::srgb(0.30, 0.26, 0.40);

const SCALE_MIN: f32 = 0.35;
const SCALE_MAX: f32 = 2.5;
const SCALE_STEP: f32 = 0.1;

#[derive(Component)]
pub struct HudRoot;
#[derive(Component)]
pub struct RosterPanel;
#[derive(Component)]
pub struct RosterRow;
#[derive(Component)]
pub struct StatusLabel;
#[derive(Component)]
pub struct HintLabel;
#[derive(Component)]
pub struct BackBtn;
#[derive(Component)]
pub struct ScaleUpBtn;
#[derive(Component)]
pub struct ScaleDownBtn;
#[derive(Component)]
pub struct DeleteBtn;
#[derive(Component)]
pub struct AssignTokenBtn(pub PlayerUuid);

#[derive(Component, Clone, Copy, PartialEq)]
pub enum ToolBtn {
    Tool(ActiveTool),
    Grid(GridKind),
    CellDelta(f32),
    Drop(DropMode),
}

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

fn tool_button(
    bar: &mut ChildSpawnerCommands,
    kind: ToolBtn,
    icon: Handle<Image>,
    si: &ScreenInfo,
) {
    let b = sz(46.0, si);
    bar.spawn((
        Button,
        kind,
        Node {
            width: Val::Px(b),
            height: Val::Px(b),
            border: UiRect::all(Val::Px(sz(2.0, si))),
            padding: UiRect::all(Val::Px(sz(6.0, si))),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BTN_BG),
        BorderColor::all(BTN_BORDER),
    ))
    .with_children(|b| {
        b.spawn((
            ImageNode::new(icon),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
    });
}

fn spawn_hud(
    commands: &mut Commands,
    assets: &GameAssets,
    session: &Session,
    si: &ScreenInfo,
    device: &DeviceProfile,
) {
    let gm = session.me.is_gm;
    let top = sz(if device.is_mobile() { 36.0 } else { 0.0 }, si);
    let bottom = sz(if device.is_mobile() { 60.0 } else { 0.0 }, si);
    let p = sz(12.0, si);
    let p2 = sz(8.0, si);
    let gap = sz(4.0, si);
    let gap2 = sz(6.0, si);
    let f0 = sz(22.0, si);
    let f1 = sz(13.0, si);
    let f2 = sz(12.0, si);
    let bw = sz(170.0, si);

    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ZIndex(50),
        ))
        .with_children(|root| {
            // left column: info panel + roster, stacked vertically
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(p + top),
                left: Val::Px(p),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(gap2),
                ..default()
            })
            .with_children(|col| {
                // topo esquerdo: sala + status
                col.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(gap),
                        padding: UiRect::all(Val::Px(p)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new(format!("SALA {}", session.code)),
                        tfont(assets, f0),
                        TextColor(GOLD),
                    ));
                    p.spawn((
                        StatusLabel,
                        Text::new("conectando..."),
                        tfont(assets, f1),
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                    p.spawn((
                        Text::new(if gm {
                            "você é o MESTRE"
                        } else {
                            "você é JOGADOR"
                        }),
                        tfont(assets, f2),
                        TextColor(Color::srgb(0.60, 0.58, 0.68)),
                    ));
                });
                // painel de jogadores
                col.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(gap2),
                        padding: UiRect::all(Val::Px(p)),
                        min_width: Val::Px(bw.min(si.width * 0.25)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("JOGADORES"),
                        tfont(assets, f2),
                        TextColor(Color::srgb(0.60, 0.58, 0.68)),
                    ));
                    p.spawn((
                        RosterPanel,
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(gap),
                            ..default()
                        },
                    ));
                });
            });
            // dica inferior direita
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(sz(70.0, si) + bottom),
                    right: Val::Px(p2),
                    padding: UiRect::all(Val::Px(p2)),
                    max_width: Val::Vw(45.0),
                    ..default()
                },
                BackgroundColor(PANEL_SOFT),
            ))
            .with_children(|p| {
                p.spawn((
                    HintLabel,
                    Text::new(""),
                    tfont(assets, f2),
                    TextColor(Color::srgb(0.75, 0.72, 0.80)),
                ));
            });
            // botão voltar ao lobby
            root.spawn((
                Button,
                BackBtn,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(p + top),
                    right: Val::Px(p),
                    padding: UiRect::all(Val::Px(sz(6.0, si))),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.30, 0.12, 0.12, 0.85)),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("SAIR"),
                    tfont(assets, sz(13.0, si)),
                    TextColor(Color::srgb(0.95, 0.70, 0.70)),
                ));
            });
            // toolbar inferior central
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(p + bottom),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                padding: UiRect::horizontal(Val::Px(p)),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|wrap| {
                // painel da toolbar + escala — quebra em linhas p/ caber em telas
                // estreitas (celular) sem escapar da tela.
                wrap.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        max_width: Val::Percent(100.0),
                        column_gap: Val::Px(gap2),
                        row_gap: Val::Px(gap2),
                        padding: UiRect::all(Val::Px(p2)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|bar| {
                    tool_button(
                        bar,
                        ToolBtn::Tool(ActiveTool::Select),
                        assets.icon("select"),
                        si,
                    );
                    if gm {
                        for i in 0..assets.textures.len() as u8 {
                            tool_button(
                                bar,
                                ToolBtn::Tool(ActiveTool::Paint(i)),
                                assets.textures[i as usize].clone(),
                                si,
                            );
                        }
                        tool_button(
                            bar,
                            ToolBtn::Tool(ActiveTool::Erase),
                            assets.icon("eraser"),
                            si,
                        );
                        tool_button(
                            bar,
                            ToolBtn::Tool(ActiveTool::ElevUp),
                            assets.icon("elev_up"),
                            si,
                        );
                        tool_button(
                            bar,
                            ToolBtn::Tool(ActiveTool::ElevDown),
                            assets.icon("elev_down"),
                            si,
                        );
                        tool_button(
                            bar,
                            ToolBtn::Grid(GridKind::Square),
                            assets.icon("grid_square"),
                            si,
                        );
                        tool_button(
                            bar,
                            ToolBtn::Grid(GridKind::HexFlat),
                            assets.icon("grid_hex"),
                            si,
                        );
                        tool_button(bar, ToolBtn::CellDelta(8.0), assets.icon("plus"), si);
                        tool_button(bar, ToolBtn::CellDelta(-8.0), assets.icon("minus"), si);
                        tool_button(bar, ToolBtn::Drop(DropMode::Map), assets.icon("map"), si);
                    }
                    tool_button(
                        bar,
                        ToolBtn::Drop(DropMode::Token),
                        assets.icon("token"),
                        si,
                    );
                    // Botão de deletar token selecionado
                    let del_bg = Color::srgba(0.45, 0.12, 0.12, 0.85);
                    bar.spawn((
                        Button,
                        DeleteBtn,
                        Node {
                            width: Val::Px(sz(46.0, si)),
                            height: Val::Px(sz(46.0, si)),
                            border: UiRect::all(Val::Px(sz(2.0, si))),
                            padding: UiRect::all(Val::Px(sz(6.0, si))),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(del_bg),
                        BorderColor::all(Color::srgb(0.6, 0.2, 0.2)),
                        Visibility::Hidden,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("X"),
                            tfont(assets, sz(18.0, si)),
                            TextColor(Color::srgb(0.95, 0.70, 0.70)),
                        ));
                    });
                    // botões de escala
                    bar.spawn((
                        Button,
                        ScaleDownBtn,
                        Node {
                            width: Val::Px(sz(32.0, si)),
                            height: Val::Px(sz(32.0, si)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::left(Val::Px(gap2)),
                            ..default()
                        },
                        BackgroundColor(BTN_BG),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("A-"),
                            tfont(assets, sz(14.0, si)),
                            TextColor(TEXT),
                        ));
                    });
                    bar.spawn((
                        Button,
                        ScaleUpBtn,
                        Node {
                            width: Val::Px(sz(32.0, si)),
                            height: Val::Px(sz(32.0, si)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(BTN_BG),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("A+"),
                            tfont(assets, sz(14.0, si)),
                            TextColor(TEXT),
                        ));
                    });
                });
            });
        });
}

pub fn scale_btn_click(
    q_down: Query<&Interaction, (Changed<Interaction>, With<ScaleDownBtn>)>,
    q_up: Query<&Interaction, (Changed<Interaction>, With<ScaleUpBtn>)>,
    mut si: ResMut<ScreenInfo>,
) {
    let delta = if q_up.iter().any(|i| *i == Interaction::Pressed) {
        SCALE_STEP
    } else if q_down.iter().any(|i| *i == Interaction::Pressed) {
        -SCALE_STEP
    } else {
        return;
    };
    let new = (si.scale + delta).clamp(SCALE_MIN, SCALE_MAX);
    if (new - si.scale).abs() < 0.001 {
        return;
    }
    si.auto_scale = false;
    si.scale = new;
    info!("escala do HUD ajustada para {new:.2}");
}

pub fn toolbar_clicks(
    q: Query<(&Interaction, &ToolBtn), Changed<Interaction>>,
    mut tool: ResMut<ActiveTool>,
    mut drop_mode: ResMut<DropMode>,
    mut grid: ResMut<GridRes>,
    mut net: ResMut<Net>,
    session: Res<Session>,
) {
    for (i, btn) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        match btn {
            ToolBtn::Tool(t) => {
                if session.me.is_gm || *t == ActiveTool::Select {
                    *tool = *t;
                }
            }
            ToolBtn::Grid(k) if session.me.is_gm => {
                if grid.0.kind != *k {
                    grid.0.kind = *k;
                    let g = grid.0;
                    net.broadcast(&Msg::Grid(g));
                }
            }
            ToolBtn::CellDelta(d) if session.me.is_gm => {
                let c = (grid.0.cell + d).clamp(24.0, 192.0);
                if c != grid.0.cell {
                    grid.0.cell = c;
                    let g = grid.0;
                    net.broadcast(&Msg::Grid(g));
                }
            }
            ToolBtn::Drop(m) => {
                *drop_mode = *m;
            }
            _ => {}
        }
    }
}

pub fn toolbar_visuals(
    mut q: Query<(&ToolBtn, &mut BorderColor, &mut BackgroundColor)>,
    tool: Res<ActiveTool>,
    drop_mode: Res<DropMode>,
    grid: Res<GridRes>,
) {
    if !(tool.is_changed() || drop_mode.is_changed() || grid.is_changed()) {
        return;
    }
    for (btn, mut border, mut bg) in &mut q {
        let active = match btn {
            ToolBtn::Tool(t) => *t == *tool,
            ToolBtn::Drop(m) => *m == *drop_mode,
            ToolBtn::Grid(k) => grid.0.kind == *k,
            ToolBtn::CellDelta(_) => false,
        };
        *border = BorderColor::all(if active { GOLD } else { BTN_BORDER });
        *bg = BackgroundColor(if active {
            Color::srgb(0.26, 0.22, 0.34)
        } else {
            BTN_BG
        });
    }
}

pub fn roster_panel(
    mut commands: Commands,
    roster: Res<Roster>,
    session: Res<Session>,
    selection: Res<TokenSelection>,
    assets: Res<GameAssets>,
    si: Res<ScreenInfo>,
    q_panel: Query<Entity, With<RosterPanel>>,
    q_rows: Query<Entity, With<RosterRow>>,
) {
    if !roster.is_changed() && !selection.is_changed() {
        return;
    }
    let Ok(_panel) = q_panel.single() else { return };
    for e in &q_rows {
        commands.entity(e).despawn();
    }
    let has_sel = session.me.is_gm && selection.0.is_some();
    for entry in &roster.list {
        let mut label = entry.meta.nick.clone();
        if entry.meta.is_gm {
            label.push_str(" [GM]");
        }
        if entry.meta.uuid == session.me.uuid {
            label.push_str(" (você)");
        }
        let col = if entry.online {
            Color::srgb(0.92, 0.90, 0.95)
        } else {
            Color::srgb(0.45, 0.43, 0.50)
        };
        let dot = sz(14.0, &si);
        let mut spawner = if has_sel && !entry.meta.is_gm {
            commands.spawn((
                Button,
                RosterRow,
                AssignTokenBtn(entry.meta.uuid),
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(sz(8.0, &si)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.20, 0.18, 0.26, 0.50)),
            ))
        } else {
            commands.spawn((
                RosterRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(sz(8.0, &si)),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
        };
        spawner.with_children(|r| {
            r.spawn((
                Node {
                    width: Val::Px(dot),
                    height: Val::Px(dot),
                    ..default()
                },
                BackgroundColor(entry.meta.color.color()),
            ));
            r.spawn((
                Text::new(label),
                tfont(&assets, sz(15.0, &si)),
                TextColor(col),
            ));
        });
    }
}

pub fn status_label(
    net: Res<Net>,
    session: Res<Session>,
    roster: Res<Roster>,
    mut q: Query<(&mut Text, &mut TextColor), With<StatusLabel>>,
) {
    let Ok((mut text, mut color)) = q.single_mut() else {
        return;
    };
    let (s, c) = if session.me.is_gm {
        let n = roster
            .list
            .iter()
            .filter(|e| e.online && !e.meta.is_gm)
            .count();
        (
            format!("{n} jogador(es) conectado(s)"),
            Color::srgb(0.55, 0.85, 0.55),
        )
    } else if net.gm_peer.is_some() {
        (
            "conectado ao mestre".to_string(),
            Color::srgb(0.55, 0.85, 0.55),
        )
    } else if net.socket.is_some() {
        (
            "procurando o mestre...".to_string(),
            Color::srgb(0.9, 0.8, 0.4),
        )
    } else {
        ("reconectando...".to_string(), Color::srgb(0.9, 0.5, 0.4))
    };
    if text.0 != s {
        text.0 = s;
        color.0 = c;
    }
}

pub fn back_btn_click(
    q: Query<&Interaction, (Changed<Interaction>, With<BackBtn>)>,
    mut next: ResMut<NextState<AppState>>,
) {
    for i in &q {
        if *i == Interaction::Pressed {
            next.set(AppState::Lobby);
        }
    }
}

pub fn assign_token_click(
    q: Query<(&Interaction, &AssignTokenBtn), Changed<Interaction>>,
    selection: Res<TokenSelection>,
    session: Res<Session>,
    mut net: ResMut<Net>,
    roster: Res<Roster>,
    mut ctx: Ctx3d,
    mut q_tokens: Query<(Entity, &mut Token, &Children)>,
    mut q_rings: Query<&mut MeshMaterial3d<StandardMaterial>, With<OwnerRing>>,
) {
    if !session.me.is_gm {
        return;
    }
    let Some(token_id) = selection.0 else { return };
    for (i, btn) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        set_token_owner(
            token_id,
            btn.0,
            &roster,
            &mut ctx,
            &mut q_tokens,
            &mut q_rings,
        );
        net.broadcast(&Msg::AssignToken {
            id: token_id,
            new_owner: btn.0,
        });
        info!("token {token_id} atribuído ao jogador {}", btn.0);
    }
}

pub fn delete_btn_visibility(
    sel: Res<TokenSelection>,
    mut q: Query<&mut Visibility, With<DeleteBtn>>,
) {
    if !sel.is_changed() {
        return;
    }
    for mut vis in &mut q {
        *vis = if sel.0.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn delete_btn_click(
    q: Query<&Interaction, (Changed<Interaction>, With<DeleteBtn>)>,
    session: Res<Session>,
    mut net: ResMut<Net>,
    mut commands: Commands,
    q_tokens: Query<(Entity, &Token)>,
    mut sel: ResMut<TokenSelection>,
) {
    for i in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        delete_selected_entity(&sel, &session, &mut net, &mut commands, &q_tokens);
        sel.0 = None;
    }
}

pub fn hint_label(
    drop_mode: Res<DropMode>,
    tool: Res<ActiveTool>,
    assets: Res<GameAssets>,
    device: Res<DeviceProfile>,
    mut q: Query<&mut Text, With<HintLabel>>,
) {
    if !(drop_mode.is_changed() || tool.is_changed()) {
        return;
    }
    let Ok(mut text) = q.single_mut() else { return };
    let mode = match *drop_mode {
        DropMode::Token => "TOKEN",
        DropMode::Map => "MAPA",
    };
    let paint_label;
    let tool_s = match *tool {
        ActiveTool::Select => "mover/selecionar",
        ActiveTool::Paint(i) => {
            let name = assets
                .tex_names
                .get(i as usize)
                .copied()
                .unwrap_or("terreno");
            paint_label = format!("pintar: {name}");
            paint_label.as_str()
        }
        ActiveTool::Erase => "apagar terreno",
        ActiveTool::ElevUp => "elevar terreno",
        ActiveTool::ElevDown => "rebaixar terreno",
    };
    if device.is_mobile() {
        text.0 = format!(
            "Ferramenta: {tool_s}  |  soltar imagem cria: {mode}\n1 dedo: mover câmera  |  2 dedos: girar  |  pinça: zoom  |  toque token: arrastar"
        );
    } else {
        text.0 = format!(
            "Ferramenta: {tool_s}  |  soltar imagem cria: {mode}\nbotão direito/WASD: mover câmera  |  botão meio/Q/E: girar  |  scroll: zoom  |  Delete: remover token  |  F12: screenshot"
        );
    }
}

pub fn hud_responsive(
    si: Res<ScreenInfo>,
    mut last: Local<(f32, f32, f32)>,
    q_root: Query<Entity, With<HudRoot>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    session: Res<Session>,
    device: Res<DeviceProfile>,
) {
    let cur = (si.width, si.height, si.scale);
    if *last == cur && !q_root.is_empty() {
        return;
    }
    *last = cur;
    for e in &q_root {
        commands.entity(e).despawn();
    }
    spawn_hud(&mut commands, &assets, &session, &si, &device);
}
