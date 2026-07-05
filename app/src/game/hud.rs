use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use super::map::DropMode;
use super::ActiveTool;
use crate::net::{Net, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::{tfont, GameAssets};

use super::grid::GridRes;

const GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
const PANEL: Color = Color::srgba(0.10, 0.09, 0.14, 0.94);
const PANEL_SOFT: Color = Color::srgba(0.10, 0.09, 0.14, 0.75);
const BTN_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const BTN_BORDER: Color = Color::srgb(0.30, 0.26, 0.40);

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

#[derive(Component, Clone, Copy, PartialEq)]
pub enum ToolBtn {
    Tool(ActiveTool),
    Grid(GridKind),
    CellDelta(f32),
    Drop(DropMode),
}

fn tool_button(bar: &mut ChildSpawnerCommands, kind: ToolBtn, icon: Handle<Image>) {
    bar.spawn((
        Button,
        kind,
        Node {
            width: Val::Px(46.0),
            height: Val::Px(46.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(6.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BTN_BG),
        BorderColor(BTN_BORDER),
        BorderRadius::all(Val::Px(8.0)),
    ))
    .with_children(|b| {
        b.spawn((
            ImageNode::new(icon),
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ));
    });
}

pub fn setup_hud(mut commands: Commands, assets: Res<GameAssets>, session: Res<Session>) {
    let gm = session.me.is_gm;
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|root| {
            // topo esquerdo: sala + status
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(12.0),
                    left: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderRadius::all(Val::Px(8.0)),
                Interaction::default(),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(format!("SALA {}", session.code)),
                    tfont(&assets, 22.0),
                    TextColor(GOLD),
                ));
                p.spawn((
                    StatusLabel,
                    Text::new("conectando..."),
                    tfont(&assets, 13.0),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
                p.spawn((
                    Text::new(if gm { "você é o MESTRE" } else { "você é JOGADOR" }),
                    tfont(&assets, 12.0),
                    TextColor(Color::srgb(0.60, 0.58, 0.68)),
                ));
            });
            // painel de jogadores
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(118.0),
                    left: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    min_width: Val::Px(170.0),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderRadius::all(Val::Px(8.0)),
                Interaction::default(),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("JOGADORES"),
                    tfont(&assets, 12.0),
                    TextColor(Color::srgb(0.60, 0.58, 0.68)),
                ));
                p.spawn((
                    RosterPanel,
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() },
                ));
            });
            // dica inferior direita
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(12.0),
                    right: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    max_width: Val::Px(420.0),
                    ..default()
                },
                BackgroundColor(PANEL_SOFT),
                BorderRadius::all(Val::Px(6.0)),
            ))
            .with_children(|p| {
                p.spawn((
                    HintLabel,
                    Text::new(""),
                    tfont(&assets, 12.0),
                    TextColor(Color::srgb(0.75, 0.72, 0.80)),
                ));
            });
            // toolbar inferior central
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|wrap| {
                wrap.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderRadius::all(Val::Px(10.0)),
                    Interaction::default(),
                ))
                .with_children(|bar| {
                    tool_button(bar, ToolBtn::Tool(ActiveTool::Select), assets.icons["select"].clone());
                    if gm {
                        for i in 0..assets.textures.len() as u8 {
                            tool_button(bar, ToolBtn::Tool(ActiveTool::Paint(i)), assets.textures[i as usize].clone());
                        }
                        tool_button(bar, ToolBtn::Tool(ActiveTool::Erase), assets.icons["eraser"].clone());
                        tool_button(bar, ToolBtn::Tool(ActiveTool::ElevUp), assets.icons["elev_up"].clone());
                        tool_button(bar, ToolBtn::Tool(ActiveTool::ElevDown), assets.icons["elev_down"].clone());
                        tool_button(bar, ToolBtn::Grid(GridKind::Square), assets.icons["grid_square"].clone());
                        tool_button(bar, ToolBtn::Grid(GridKind::HexFlat), assets.icons["grid_hex"].clone());
                        tool_button(bar, ToolBtn::CellDelta(8.0), assets.icons["plus"].clone());
                        tool_button(bar, ToolBtn::CellDelta(-8.0), assets.icons["minus"].clone());
                        tool_button(bar, ToolBtn::Drop(DropMode::Map), assets.icons["map"].clone());
                    }
                    tool_button(bar, ToolBtn::Drop(DropMode::Token), assets.icons["token"].clone());
                });
            });
        });
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
        border.0 = if active { GOLD } else { BTN_BORDER };
        *bg = BackgroundColor(if active { Color::srgb(0.26, 0.22, 0.34) } else { BTN_BG });
    }
}

pub fn roster_panel(
    mut commands: Commands,
    roster: Res<Roster>,
    session: Res<Session>,
    assets: Res<GameAssets>,
    q_panel: Query<Entity, With<RosterPanel>>,
    q_rows: Query<Entity, With<RosterRow>>,
) {
    if !roster.is_changed() {
        return;
    }
    let Ok(panel) = q_panel.single() else { return };
    for e in &q_rows {
        commands.entity(e).despawn();
    }
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
        let row = commands
            .spawn((
                RosterRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|r| {
                r.spawn((
                    Node { width: Val::Px(14.0), height: Val::Px(14.0), ..default() },
                    BackgroundColor(palette_color(entry.meta.color)),
                    BorderRadius::all(Val::Px(7.0)),
                ));
                r.spawn((Text::new(label), tfont(&assets, 15.0), TextColor(col)));
            })
            .id();
        commands.entity(panel).add_child(row);
    }
}

pub fn status_label(
    net: Res<Net>,
    session: Res<Session>,
    roster: Res<Roster>,
    mut q: Query<(&mut Text, &mut TextColor), With<StatusLabel>>,
) {
    let Ok((mut text, mut color)) = q.single_mut() else { return };
    let (s, c) = if session.me.is_gm {
        let n = roster.list.iter().filter(|e| e.online && !e.meta.is_gm).count();
        (format!("{n} jogador(es) conectado(s)"), Color::srgb(0.55, 0.85, 0.55))
    } else if net.gm_peer.is_some() {
        ("conectado ao mestre".to_string(), Color::srgb(0.55, 0.85, 0.55))
    } else if net.socket.is_some() {
        ("procurando o mestre...".to_string(), Color::srgb(0.9, 0.8, 0.4))
    } else {
        ("reconectando...".to_string(), Color::srgb(0.9, 0.5, 0.4))
    };
    if text.0 != s {
        text.0 = s;
        color.0 = c;
    }
}

pub fn hint_label(
    drop_mode: Res<DropMode>,
    tool: Res<ActiveTool>,
    assets: Res<GameAssets>,
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
            let name = assets.tex_names.get(i as usize).copied().unwrap_or("terreno");
            paint_label = format!("pintar: {name}");
            paint_label.as_str()
        }
        ActiveTool::Erase => "apagar terreno",
        ActiveTool::ElevUp => "elevar terreno",
        ActiveTool::ElevDown => "rebaixar terreno",
    };
    text.0 = format!(
        "Ferramenta: {tool_s}  |  soltar imagem cria: {mode}\nbotão direito/WASD: mover câmera  |  botão meio/Q/E: girar  |  scroll: zoom  |  Delete: remover token  |  F12: screenshot"
    );
}
