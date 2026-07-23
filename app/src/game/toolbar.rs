//! Barra de tarefas modular — o framework de ferramentas da interface (#18).
//!
//! Segue o padrão de `graphics.rs`: as ferramentas são **dados** (uma lista de
//! `ToolItem`), não `dyn` trait objects. Adicionar uma tool = adicionar um item
//! em `tools()`; o container abaixo renderiza o que estiver registrado, sem
//! precisar ser editado. Posição adaptável ao dispositivo (P0.1): faixa inferior
//! em paisagem, coluna lateral em retrato. Ícones vêm do sistema SVG→PNG (P0.2).

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use super::grid::GridRes;
use super::map::DropMode;
use super::tokens::{delete_selected_entity, Selection as TokenSelection, Token};
use super::{ActiveTool, ScreenInfo};
use crate::net::{Net, Session};
use crate::protocol::*;
use crate::svg_assets::{tfont, GameAssets};
use crate::DeviceProfile;

const GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
const PANEL: Color = Color::srgba(0.10, 0.09, 0.14, 0.94);
const BTN_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const BTN_BORDER: Color = Color::srgb(0.30, 0.26, 0.40);
const ACTIVE_BG: Color = Color::srgb(0.26, 0.22, 0.34);

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

// ─── Componentes ─────────────────────────────────────────────────────────────

/// Raiz do container da toolbar (root próprio, separado do HudRoot).
#[derive(Component)]
pub struct ToolbarRoot;

/// Painel de opções de uma tool (ex.: terreno → texturas/elevação). Aparece só
/// quando a tool correspondente está ativa (via `toolbar_options_visibility`).
#[derive(Component)]
pub struct ToolOptionsPanel;

/// Botão de deletar o token selecionado (ação contextual de ferramenta).
#[derive(Component)]
pub struct DeleteBtn;

/// Descreve o que um botão da toolbar faz ao ser clicado. Reaproveitado pelos
/// sistemas de clique, atalho e destaque.
#[derive(Component, Clone, Copy, PartialEq)]
pub enum ToolBtn {
    /// Define a ferramenta ativa (radio). `Select` é liberado a todos.
    Tool(ActiveTool),
    /// Botão-mãe do grupo Terreno: entra no modo terreno e revela as opções.
    TerrainGroup,
    /// Troca o tipo de grade (GM).
    Grid(GridKind),
    /// Ajusta o tamanho da célula (GM).
    CellDelta(f32),
    /// Define o modo de soltar imagem (token/mapa).
    Drop(DropMode),
}

// ─── Registro data-driven (a "interface de Tool") ────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    All,
    GmOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Group {
    /// Botão fica na barra principal.
    Main,
    /// Botão fica no painel de opções de terreno.
    TerrainOpts,
}

struct ToolItem {
    btn: ToolBtn,
    icon: Handle<Image>,
    role: Role,
    group: Group,
}

/// Fonte única da toolbar. Adicionar uma ferramenta é adicionar um item aqui —
/// o container renderiza automaticamente, sem ser editado (Critério de Aceite 1).
fn tools(assets: &GameAssets) -> Vec<ToolItem> {
    let mut v = vec![
        ToolItem {
            btn: ToolBtn::Tool(ActiveTool::Select),
            icon: assets.icon("select"),
            role: Role::All,
            group: Group::Main,
        },
        // Botão-mãe do terreno usa a 1ª textura como ícone ("pintar").
        ToolItem {
            btn: ToolBtn::TerrainGroup,
            icon: assets.textures.first().cloned().unwrap_or_default(),
            role: Role::GmOnly,
            group: Group::Main,
        },
        ToolItem {
            btn: ToolBtn::Grid(GridKind::Square),
            icon: assets.icon("grid_square"),
            role: Role::GmOnly,
            group: Group::Main,
        },
        ToolItem {
            btn: ToolBtn::Grid(GridKind::HexFlat),
            icon: assets.icon("grid_hex"),
            role: Role::GmOnly,
            group: Group::Main,
        },
        ToolItem {
            btn: ToolBtn::CellDelta(8.0),
            icon: assets.icon("plus"),
            role: Role::GmOnly,
            group: Group::Main,
        },
        ToolItem {
            btn: ToolBtn::CellDelta(-8.0),
            icon: assets.icon("minus"),
            role: Role::GmOnly,
            group: Group::Main,
        },
        ToolItem {
            btn: ToolBtn::Drop(DropMode::Map),
            icon: assets.icon("map"),
            role: Role::GmOnly,
            group: Group::Main,
        },
        ToolItem {
            btn: ToolBtn::Drop(DropMode::Token),
            icon: assets.icon("token"),
            role: Role::All,
            group: Group::Main,
        },
    ];
    // Opções de terreno: uma textura por swatch + apagar + elevar/rebaixar.
    for i in 0..assets.textures.len() as u8 {
        v.push(ToolItem {
            btn: ToolBtn::Tool(ActiveTool::Paint(i)),
            icon: assets.textures[i as usize].clone(),
            role: Role::GmOnly,
            group: Group::TerrainOpts,
        });
    }
    v.push(ToolItem {
        btn: ToolBtn::Tool(ActiveTool::Erase),
        icon: assets.icon("eraser"),
        role: Role::GmOnly,
        group: Group::TerrainOpts,
    });
    v.push(ToolItem {
        btn: ToolBtn::Tool(ActiveTool::ElevUp),
        icon: assets.icon("elev_up"),
        role: Role::GmOnly,
        group: Group::TerrainOpts,
    });
    v.push(ToolItem {
        btn: ToolBtn::Tool(ActiveTool::ElevDown),
        icon: assets.icon("elev_down"),
        role: Role::GmOnly,
        group: Group::TerrainOpts,
    });
    v
}

/// Atalhos de teclado (desktop): mesma ação do clique no botão correspondente.
const SHORTCUTS: &[(KeyCode, ToolBtn)] = &[
    (KeyCode::Digit1, ToolBtn::Tool(ActiveTool::Select)),
    (KeyCode::Digit2, ToolBtn::TerrainGroup),
    (KeyCode::Digit3, ToolBtn::Tool(ActiveTool::Erase)),
    (KeyCode::Digit4, ToolBtn::Tool(ActiveTool::ElevUp)),
    (KeyCode::Digit5, ToolBtn::Tool(ActiveTool::ElevDown)),
];

fn is_terrain(t: ActiveTool) -> bool {
    matches!(
        t,
        ActiveTool::Paint(_) | ActiveTool::Erase | ActiveTool::ElevUp | ActiveTool::ElevDown
    )
}

fn role_allowed(role: Role, gm: bool) -> bool {
    gm || role == Role::All
}

// ─── Construção da UI ────────────────────────────────────────────────────────

fn tool_button(bar: &mut ChildSpawnerCommands, btn: ToolBtn, icon: Handle<Image>, si: &ScreenInfo) {
    let b = sz(46.0, si);
    bar.spawn((
        Button,
        btn,
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

fn spawn_delete_button(bar: &mut ChildSpawnerCommands, assets: &GameAssets, si: &ScreenInfo) {
    bar.spawn((
        Button,
        DeleteBtn,
        Node {
            // Oculto via Display::None p/ não reservar slot na coluna quando não
            // há token selecionado (senão a barra estoura a banda e quebra em 2).
            display: Display::None,
            width: Val::Px(sz(46.0, si)),
            height: Val::Px(sz(46.0, si)),
            border: UiRect::all(Val::Px(sz(2.0, si))),
            padding: UiRect::all(Val::Px(sz(6.0, si))),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.45, 0.12, 0.12, 0.85)),
        BorderColor::all(Color::srgb(0.6, 0.2, 0.2)),
    ))
    .with_children(|b| {
        b.spawn((
            Text::new("X"),
            tfont(assets, sz(18.0, si)),
            TextColor(Color::srgb(0.95, 0.70, 0.70)),
        ));
    });
}

fn spawn_toolbar_inner(
    commands: &mut Commands,
    assets: &GameAssets,
    session: &Session,
    si: &ScreenInfo,
    device: &DeviceProfile,
) {
    let gm = session.me.is_gm;
    let portrait = si.height > si.width;
    let p = sz(8.0, si);
    let gap = sz(6.0, si);
    // Margens p/ não colidir com as barras do sistema Android.
    let bottom = sz(if device.is_mobile() { 60.0 } else { 0.0 }, si);

    // Root ancorado conforme orientação: paisagem = faixa inferior central;
    // retrato = coluna na lateral direita.
    let root = if portrait {
        // Ancorada por top+bottom (não centralizada em 100% da altura): fica
        // sempre ABAIXO dos controles do topo (SAIR/escala/Gráficos) e ACIMA da
        // zona reservada aos joysticks (#24), em qualquer escala — se centralizada,
        // a coluna sobe atrás dos controles quando a escala cresce (Select some).
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(p),
            top: Val::Px(sz(190.0, si)),
            bottom: Val::Px(p + bottom),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(gap),
            ..default()
        }
    } else {
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(p + bottom),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(gap),
            ..default()
        }
    };

    let dir = if portrait {
        FlexDirection::Column
    } else {
        FlexDirection::Row
    };
    let cap = |n: &mut Node| {
        if portrait {
            // 8 botões não cabem em 1 coluna nesta tela; quebra intencional em
            // 2 colunas de até 4 (altura de 4 botões: 4*46 + 3*gap + 2*pad).
            n.max_height = Val::Px(sz(220.0, si));
        } else {
            n.max_width = Val::Vw(96.0);
        }
    };

    commands
        .spawn((ToolbarRoot, root, ZIndex(50)))
        .with_children(|root| {
            // Painel de opções de terreno (só GM). Começa oculto via Display::None
            // — assim não reserva espaço quando o terreno não está ativo.
            if gm {
                let mut opts = Node {
                    display: Display::None,
                    flex_direction: dir,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(gap),
                    row_gap: Val::Px(gap),
                    padding: UiRect::all(Val::Px(p)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                };
                cap(&mut opts);
                root.spawn((ToolOptionsPanel, opts, BackgroundColor(PANEL)))
                    .with_children(|panel| {
                        for it in tools(assets) {
                            if it.group == Group::TerrainOpts && role_allowed(it.role, gm) {
                                tool_button(panel, it.btn, it.icon, si);
                            }
                        }
                    });
            }

            // Barra principal.
            let mut bar = Node {
                flex_direction: dir,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(gap),
                row_gap: Val::Px(gap),
                padding: UiRect::all(Val::Px(p)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            };
            cap(&mut bar);
            root.spawn((bar, BackgroundColor(PANEL)))
                .with_children(|bar| {
                    for it in tools(assets) {
                        if it.group == Group::Main && role_allowed(it.role, gm) {
                            tool_button(bar, it.btn, it.icon, si);
                        }
                    }
                    spawn_delete_button(bar, assets, si);
                });
        });
}

// ─── Sistemas ────────────────────────────────────────────────────────────────

/// Aplica o efeito de um `ToolBtn` — compartilhado por clique e atalho.
fn apply_tool_btn(
    btn: ToolBtn,
    gm: bool,
    tool: &mut ActiveTool,
    drop_mode: &mut DropMode,
    grid: &mut GridRes,
    net: &mut Net,
) {
    match btn {
        ToolBtn::Tool(t) => {
            if gm || t == ActiveTool::Select {
                *tool = t;
            }
        }
        ToolBtn::TerrainGroup => {
            if gm {
                *tool = ActiveTool::Paint(0);
            }
        }
        ToolBtn::Grid(k) if gm => {
            if grid.0.kind != k {
                grid.0.kind = k;
                let g = grid.0;
                net.broadcast(&Msg::Grid(g));
            }
        }
        ToolBtn::CellDelta(d) if gm => {
            let c = (grid.0.cell + d).clamp(24.0, 192.0);
            if c != grid.0.cell {
                grid.0.cell = c;
                let g = grid.0;
                net.broadcast(&Msg::Grid(g));
            }
        }
        ToolBtn::Drop(m) => {
            *drop_mode = m;
        }
        _ => {}
    }
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
        if *i == Interaction::Pressed {
            apply_tool_btn(
                *btn,
                session.me.is_gm,
                &mut tool,
                &mut drop_mode,
                &mut grid,
                &mut net,
            );
        }
    }
}

pub fn toolbar_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool: ResMut<ActiveTool>,
    mut drop_mode: ResMut<DropMode>,
    mut grid: ResMut<GridRes>,
    mut net: ResMut<Net>,
    session: Res<Session>,
) {
    for (k, btn) in SHORTCUTS {
        if keys.just_pressed(*k) {
            apply_tool_btn(
                *btn,
                session.me.is_gm,
                &mut tool,
                &mut drop_mode,
                &mut grid,
                &mut net,
            );
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
            ToolBtn::TerrainGroup => is_terrain(*tool),
            ToolBtn::Drop(m) => *m == *drop_mode,
            ToolBtn::Grid(k) => grid.0.kind == *k,
            ToolBtn::CellDelta(_) => false,
        };
        *border = BorderColor::all(if active { GOLD } else { BTN_BORDER });
        *bg = BackgroundColor(if active { ACTIVE_BG } else { BTN_BG });
    }
}

/// Mostra o painel de opções de terreno só quando a tool ativa é de terreno.
/// Usa `Display` (não `Visibility`) para não reservar espaço quando oculto.
pub fn toolbar_options_visibility(
    tool: Res<ActiveTool>,
    mut q: Query<&mut Node, With<ToolOptionsPanel>>,
    q_new: Query<(), Added<ToolOptionsPanel>>,
) {
    // `q_new` garante o estado correto quando a toolbar é reconstruída.
    if !tool.is_changed() && q_new.is_empty() {
        return;
    }
    let show = is_terrain(*tool);
    for mut n in &mut q {
        n.display = if show { Display::Flex } else { Display::None };
    }
}

pub fn delete_btn_visibility(
    sel: Res<TokenSelection>,
    mut q: Query<&mut Node, With<DeleteBtn>>,
    q_new: Query<(), Added<DeleteBtn>>,
) {
    // `q_new` reavalia quando a toolbar é reconstruída (rotação/escala respawnam
    // o botão oculto) — senão o X sumia mesmo com um token selecionado.
    if !sel.is_changed() && q_new.is_empty() {
        return;
    }
    // Display (não Visibility): oculto não deve reservar slot na coluna.
    for mut n in &mut q {
        n.display = if sel.0.is_some() {
            Display::Flex
        } else {
            Display::None
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

/// Reconstrói a toolbar quando a escala OU a orientação muda (padrão #43:
/// respawnar só nesses eventos evita o flicker de content-rect no Android).
/// Também faz o spawn inicial (quando não existe root ainda).
pub fn toolbar_responsive(
    si: Res<ScreenInfo>,
    mut last: Local<(f32, bool)>,
    q_root: Query<Entity, With<ToolbarRoot>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    session: Res<Session>,
    device: Res<DeviceProfile>,
) {
    let portrait = si.height > si.width;
    if !q_root.is_empty() && (si.scale - last.0).abs() < 0.01 && portrait == last.1 {
        return;
    }
    *last = (si.scale, portrait);
    for e in &q_root {
        commands.entity(e).despawn();
    }
    spawn_toolbar_inner(&mut commands, &assets, &session, &si, &device);
}

pub fn despawn_toolbar(mut commands: Commands, q: Query<Entity, With<ToolbarRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
