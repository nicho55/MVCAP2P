//! Toolbar do jogo (#49) — barra **inferior central**, horizontal, com botões
//! circulares (moldura SVG + ícone PNG). Segue os mockups aprovados
//! (`docs/layouts/mockup-toolbar-expanded.svg`):
//!
//! - até **4 tools visíveis** por página; carousel `◄` + dots para o resto;
//! - tocar uma tool com **submenu** abre a lista **para cima** (ex.: Terreno →
//!   pinturas/elevação/borracha; Grade → quadrada/hex/célula±);
//! - **GM** vê edição de terreno/grade; **jogador** vê só seleção + token.
//!
//! A UI é reconstruída a partir do estado (`ToolbarState`) — página e submenu
//! aberto — sempre que esse estado, a escala ou a orientação mudam (padrão dos
//! demais `*_responsive`, seguro para o Android, ver #43). O destaque radio da
//! tool ativa é atualizado ao vivo por `toolbar_visuals`, sem respawn.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use super::grid::GridRes;
use super::map::DropMode;
use super::{ActiveTool, ScreenInfo};
use crate::net::{Net, Session};
use crate::protocol::*;
use crate::svg_assets::{tfont, GameAssets};
use crate::DeviceProfile;

const TRACK: Color = Color::srgba(0.11, 0.10, 0.16, 0.92);
const BTN_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const BTN_BORDER: Color = Color::srgb(0.40, 0.40, 0.80);
const ACTIVE_BG: Color = Color::srgb(0.23, 0.23, 0.48);
const ACTIVE_BORDER: Color = Color::srgb(0.53, 0.53, 1.0);
const SUB_BG: Color = Color::srgba(0.16, 0.16, 0.29, 0.96);
const DOT: Color = Color::srgb(0.33, 0.33, 0.36);
const DOT_ON: Color = Color::srgb(0.53, 0.53, 1.0);
const TEXT: Color = Color::srgb(0.86, 0.86, 0.95);

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

/// Quantas tools cabem por página do carousel.
const PER_PAGE: usize = 4;

// ─── Modelo ──────────────────────────────────────────────────────────────────

/// Ferramenta primária (um botão do carousel). Pode ou não ter submenu.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    Select,
    Terrain,
    Grid,
    Token,
    Map,
}

struct PrimDef {
    prim: Prim,
    gm_only: bool,
}

/// Fonte única da ordem das tools. Adicionar uma primária = adicionar um item.
const PRIMS: [PrimDef; 5] = [
    PrimDef {
        prim: Prim::Select,
        gm_only: false,
    },
    PrimDef {
        prim: Prim::Terrain,
        gm_only: true,
    },
    PrimDef {
        prim: Prim::Grid,
        gm_only: true,
    },
    PrimDef {
        prim: Prim::Token,
        gm_only: false,
    },
    PrimDef {
        prim: Prim::Map,
        gm_only: true,
    },
];

/// Ação de um item de submenu.
#[derive(Clone, Copy)]
enum SubAct {
    Tool(ActiveTool),
    Grid(GridKind),
    Cell(f32),
}

/// Estado do carousel/submenu — sobrevive ao respawn da UI.
#[derive(Resource, Default)]
pub struct ToolbarState {
    pub page: usize,
    pub open: Option<Prim>,
}

// ─── Componentes ─────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ToolbarRoot;
#[derive(Component, Clone, Copy)]
pub(crate) struct PrimBtn(Prim);
#[derive(Component, Clone, Copy)]
pub(crate) struct SubBtn(SubAct);
#[derive(Component)]
pub(crate) struct CarouselPrev;

// ─── Registro de ícones/labels ───────────────────────────────────────────────

fn prim_icon(p: Prim, a: &GameAssets) -> Handle<Image> {
    match p {
        Prim::Select => a.icon("select"),
        Prim::Terrain => a.textures.first().cloned().unwrap_or_default(),
        Prim::Grid => a.icon("grid_square"),
        Prim::Token => a.icon("token"),
        Prim::Map => a.icon("map"),
    }
}

fn prim_has_menu(p: Prim) -> bool {
    matches!(p, Prim::Terrain | Prim::Grid)
}

/// Itens do submenu de uma primária: (ícone, label, ação).
fn submenu(p: Prim, a: &GameAssets) -> Vec<(Handle<Image>, &'static str, SubAct)> {
    match p {
        Prim::Terrain => {
            let mut v = Vec::new();
            for i in 0..a.textures.len() as u8 {
                let name = a.tex_names.get(i as usize).copied().unwrap_or("Terreno");
                v.push((
                    a.textures[i as usize].clone(),
                    name,
                    SubAct::Tool(ActiveTool::Paint(i)),
                ));
            }
            v.push((
                a.icon("elev_up"),
                "Elevar",
                SubAct::Tool(ActiveTool::ElevUp),
            ));
            v.push((
                a.icon("elev_down"),
                "Rebaixar",
                SubAct::Tool(ActiveTool::ElevDown),
            ));
            v.push((
                a.icon("eraser"),
                "Borracha",
                SubAct::Tool(ActiveTool::Erase),
            ));
            v
        }
        Prim::Grid => vec![
            (
                a.icon("grid_square"),
                "Quadrada",
                SubAct::Grid(GridKind::Square),
            ),
            (
                a.icon("grid_hex"),
                "Hexagonal",
                SubAct::Grid(GridKind::HexFlat),
            ),
            (a.icon("plus"), "Célula +", SubAct::Cell(8.0)),
            (a.icon("minus"), "Célula −", SubAct::Cell(-8.0)),
        ],
        _ => Vec::new(),
    }
}

fn is_terrain(t: ActiveTool) -> bool {
    matches!(
        t,
        ActiveTool::Paint(_) | ActiveTool::Erase | ActiveTool::ElevUp | ActiveTool::ElevDown
    )
}

// ─── Construção da UI ────────────────────────────────────────────────────────

fn circle_button(
    track: &mut ChildSpawnerCommands,
    comp: PrimBtn,
    icon: Handle<Image>,
    si: &ScreenInfo,
) {
    let d = sz(46.0, si);
    track
        .spawn((
            Button,
            comp,
            Node {
                width: Val::Px(d),
                height: Val::Px(d),
                border: UiRect::all(Val::Px(sz(2.0, si))),
                padding: UiRect::all(Val::Px(sz(8.0, si))),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Percent(50.0)),
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

fn sub_row(
    panel: &mut ChildSpawnerCommands,
    act: SubAct,
    icon: Handle<Image>,
    label: &str,
    assets: &GameAssets,
    si: &ScreenInfo,
) {
    panel
        .spawn((
            Button,
            SubBtn(act),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(sz(8.0, si)),
                width: Val::Px(sz(150.0, si)),
                height: Val::Px(sz(30.0, si)),
                padding: UiRect::horizontal(Val::Px(sz(8.0, si))),
                border: UiRect::all(Val::Px(sz(1.0, si))),
                border_radius: BorderRadius::all(Val::Px(sz(6.0, si))),
                ..default()
            },
            BackgroundColor(BTN_BG),
            BorderColor::all(BTN_BORDER),
        ))
        .with_children(|r| {
            r.spawn((
                ImageNode::new(icon),
                Node {
                    width: Val::Px(sz(18.0, si)),
                    height: Val::Px(sz(18.0, si)),
                    ..default()
                },
            ));
            r.spawn((
                Text::new(label.to_string()),
                tfont(assets, sz(12.0, si)),
                TextColor(TEXT),
            ));
        });
}

fn spawn_toolbar(
    commands: &mut Commands,
    assets: &GameAssets,
    session: &Session,
    si: &ScreenInfo,
    device: &DeviceProfile,
    state: &ToolbarState,
) {
    let gm = session.me.is_gm;
    let vis: Vec<Prim> = PRIMS
        .iter()
        .filter(|d| gm || !d.gm_only)
        .map(|d| d.prim)
        .collect();
    if vis.is_empty() {
        return;
    }
    let pages = vis.len().div_ceil(PER_PAGE).max(1);
    let page = state.page % pages;
    let start = page * PER_PAGE;
    let end = (start + PER_PAGE).min(vis.len());
    let slice = &vis[start..end];

    let gap = sz(8.0, si);
    let bottom = sz(if device.is_mobile() { 56.0 } else { 14.0 }, si);

    // Só mostra o submenu se a primária aberta estiver visível para este papel.
    let open = state.open.filter(|p| vis.contains(p) && prim_has_menu(*p));

    commands
        .spawn((
            ToolbarRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(bottom),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(sz(6.0, si)),
                ..default()
            },
            ZIndex(50),
        ))
        .with_children(|root| {
            // Submenu, acima da barra (abre para cima).
            if let Some(p) = open {
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        row_gap: Val::Px(sz(4.0, si)),
                        padding: UiRect::all(Val::Px(sz(8.0, si))),
                        border_radius: BorderRadius::all(Val::Px(sz(8.0, si))),
                        ..default()
                    },
                    BackgroundColor(SUB_BG),
                ))
                .with_children(|panel| {
                    for (icon, label, act) in submenu(p, assets) {
                        sub_row(panel, act, icon, label, assets, si);
                    }
                });
            }

            // Dots de página do carousel.
            if pages > 1 {
                root.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(sz(5.0, si)),
                    ..default()
                })
                .with_children(|dots| {
                    for i in 0..pages {
                        dots.spawn((
                            Node {
                                width: Val::Px(sz(5.0, si)),
                                height: Val::Px(sz(5.0, si)),
                                border_radius: BorderRadius::all(Val::Percent(50.0)),
                                ..default()
                            },
                            BackgroundColor(if i == page { DOT_ON } else { DOT }),
                        ));
                    }
                });
            }

            // Track horizontal: tools + seta do carousel.
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(gap),
                    padding: UiRect::all(Val::Px(sz(8.0, si))),
                    border_radius: BorderRadius::all(Val::Px(sz(16.0, si))),
                    ..default()
                },
                BackgroundColor(TRACK),
            ))
            .with_children(|track| {
                for p in slice {
                    circle_button(track, PrimBtn(*p), prim_icon(*p, assets), si);
                }
                if pages > 1 {
                    track
                        .spawn((
                            Button,
                            CarouselPrev,
                            Node {
                                width: Val::Px(sz(30.0, si)),
                                height: Val::Px(sz(46.0, si)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("◄"),
                                tfont(assets, sz(16.0, si)),
                                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                            ));
                        });
                }
            });
        });
}

// ─── Sistemas ────────────────────────────────────────────────────────────────

/// Reconstrói a toolbar quando o estado (página/submenu), a escala ou a
/// orientação mudam. Também faz o spawn inicial.
pub fn toolbar_build(
    si: Res<ScreenInfo>,
    state: Res<ToolbarState>,
    mut last: Local<Option<(u32, bool, usize, u8)>>,
    q_root: Query<Entity, With<ToolbarRoot>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    session: Res<Session>,
    device: Res<DeviceProfile>,
) {
    let portrait = si.height > si.width;
    let open_code = match state.open {
        None => 0,
        Some(Prim::Select) => 1,
        Some(Prim::Terrain) => 2,
        Some(Prim::Grid) => 3,
        Some(Prim::Token) => 4,
        Some(Prim::Map) => 5,
    };
    let key = ((si.scale * 100.0) as u32, portrait, state.page, open_code);
    if !q_root.is_empty() && *last == Some(key) {
        return;
    }
    *last = Some(key);
    for e in &q_root {
        commands.entity(e).despawn();
    }
    spawn_toolbar(&mut commands, &assets, &session, &si, &device, &state);
}

/// Cliques nas tools, itens de submenu e seta do carousel.
pub fn toolbar_click(
    q_prim: Query<(&Interaction, &PrimBtn), Changed<Interaction>>,
    q_sub: Query<(&Interaction, &SubBtn), Changed<Interaction>>,
    q_prev: Query<&Interaction, (Changed<Interaction>, With<CarouselPrev>)>,
    mut state: ResMut<ToolbarState>,
    mut tool: ResMut<ActiveTool>,
    mut drop_mode: ResMut<DropMode>,
    mut grid: ResMut<GridRes>,
    mut net: ResMut<Net>,
    session: Res<Session>,
) {
    let gm = session.me.is_gm;
    for (i, b) in &q_prim {
        if *i != Interaction::Pressed {
            continue;
        }
        match b.0 {
            Prim::Select => {
                *tool = ActiveTool::Select;
                state.open = None;
            }
            Prim::Token => {
                *drop_mode = DropMode::Token;
                state.open = None;
            }
            Prim::Map => {
                if gm {
                    *drop_mode = DropMode::Map;
                }
                state.open = None;
            }
            Prim::Terrain if gm => {
                if state.open == Some(Prim::Terrain) {
                    state.open = None;
                } else {
                    state.open = Some(Prim::Terrain);
                    *tool = ActiveTool::Paint(0);
                }
            }
            Prim::Grid if gm => {
                state.open = if state.open == Some(Prim::Grid) {
                    None
                } else {
                    Some(Prim::Grid)
                };
            }
            _ => {}
        }
    }
    for (i, b) in &q_sub {
        if *i != Interaction::Pressed {
            continue;
        }
        match b.0 {
            SubAct::Tool(t) if gm => *tool = t,
            SubAct::Grid(k) if gm => {
                if grid.0.kind != k {
                    grid.0.kind = k;
                    let g = grid.0;
                    net.broadcast(&Msg::Grid(g));
                }
            }
            SubAct::Cell(d) if gm => {
                let c = (grid.0.cell + d).clamp(24.0, 192.0);
                if c != grid.0.cell {
                    grid.0.cell = c;
                    let g = grid.0;
                    net.broadcast(&Msg::Grid(g));
                }
            }
            _ => {}
        }
    }
    if q_prev.iter().any(|i| *i == Interaction::Pressed) {
        state.page = state.page.wrapping_add(1);
    }
}

fn set_active(bg: &mut BackgroundColor, bd: &mut BorderColor, on: bool) {
    *bg = BackgroundColor(if on { ACTIVE_BG } else { BTN_BG });
    *bd = BorderColor::all(if on { ACTIVE_BORDER } else { BTN_BORDER });
}

/// Destaque radio da tool/submenu ativos (ao vivo, sem respawn).
pub fn toolbar_visuals(
    tool: Res<ActiveTool>,
    drop_mode: Res<DropMode>,
    grid: Res<GridRes>,
    state: Res<ToolbarState>,
    q_added: Query<(), Added<PrimBtn>>,
    mut q_prim: Query<(&PrimBtn, &mut BackgroundColor, &mut BorderColor), Without<SubBtn>>,
    mut q_sub: Query<(&SubBtn, &mut BackgroundColor, &mut BorderColor), Without<PrimBtn>>,
) {
    if !(tool.is_changed()
        || drop_mode.is_changed()
        || grid.is_changed()
        || state.is_changed()
        || !q_added.is_empty())
    {
        return;
    }
    for (b, mut bg, mut bd) in &mut q_prim {
        let active = match b.0 {
            Prim::Select => *tool == ActiveTool::Select,
            Prim::Terrain => is_terrain(*tool),
            Prim::Grid => state.open == Some(Prim::Grid),
            Prim::Token => *drop_mode == DropMode::Token,
            Prim::Map => *drop_mode == DropMode::Map,
        };
        set_active(&mut bg, &mut bd, active);
    }
    for (b, mut bg, mut bd) in &mut q_sub {
        let active = match b.0 {
            SubAct::Tool(t) => *tool == t,
            SubAct::Grid(k) => grid.0.kind == k,
            SubAct::Cell(_) => false,
        };
        set_active(&mut bg, &mut bd, active);
    }
}

pub fn despawn_toolbar(mut commands: Commands, q: Query<Entity, With<ToolbarRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
