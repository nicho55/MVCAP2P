//! Inspector (#19) — componente **(1)** do layout: barra fina no **topo centro**
//! que resume o token selecionado e, ao toque, **expande para baixo** mostrando
//! os dados da peça. Segue o mockup aprovado (`docs/layouts/mockup-inspector-
//! expanded.svg`), tema verde.
//!
//! O modelo atual de token (`TokenMeta`: dono, arte, célula) não tem ficha de
//! RPG (HP/atributos/habilidades), então o inspector mostra o que existe de
//! fato: arte, tipo, dono/relação e posição. A ficha completa depende de um
//! modelo de stats em rede (issue à parte) — ver PR do #19.
//!
//! Reconstruído a partir do estado sempre que a seleção, a arte/dono/posição do
//! token, o modo expandido ou a escala mudam (respawn atômico, sem flicker,
//! mesmo padrão do `toolbar_build`; seguro no Android, ver #43).

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use super::tokens::{Selection, Token};
use super::ScreenInfo;
use crate::net::{Roster, Session};
use crate::protocol::*;
use crate::svg_assets::{tfont, GameAssets};
use crate::DeviceProfile;

const BG: Color = Color::srgba(0.11, 0.17, 0.11, 0.94);
const BORDER: Color = Color::srgb(0.27, 0.67, 0.27);
const SEC: Color = Color::srgb(0.40, 0.80, 0.40);
const LBL: Color = Color::srgb(0.53, 0.80, 0.53);
const VAL: Color = Color::srgb(0.96, 0.96, 0.96);

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

/// Recolhido (barra) vs expandido (ficha).
#[derive(Resource, Default)]
pub struct InspectorState {
    pub expanded: bool,
}

#[derive(Component)]
pub struct InspectorRoot;
/// A barra inteira é o handle de expandir/recolher.
#[derive(Component)]
pub struct InspectorBar;

fn art_thumb(art: TokenArt, a: &GameAssets) -> Handle<Image> {
    match art {
        TokenArt::BuiltIn(i) => a
            .tokens_builtin
            .get(i as usize)
            .cloned()
            .unwrap_or_default(),
        TokenArt::Blob(_) => a.icon("token"),
    }
}

fn art_label(art: TokenArt) -> &'static str {
    match art {
        TokenArt::BuiltIn(0) => "Guerreiro",
        TokenArt::BuiltIn(1) => "Mago",
        TokenArt::BuiltIn(2) => "Ladino",
        TokenArt::BuiltIn(3) => "Dragão",
        TokenArt::BuiltIn(_) => "Token",
        TokenArt::Blob(_) => "Imagem personalizada",
    }
}

/// Rótulo do dono a partir do roster: "você", nick (+ "[GM]") ou "sem dono".
fn owner_label(owner: PlayerUuid, roster: &Roster, session: &Session) -> String {
    if owner == session.me.uuid {
        return "você".to_string();
    }
    match roster.list.iter().find(|e| e.meta.uuid == owner) {
        Some(e) if e.meta.is_gm => format!("{} [GM]", e.meta.nick),
        Some(e) => e.meta.nick.clone(),
        None => "sem dono".to_string(),
    }
}

fn owner_color(owner: PlayerUuid, roster: &Roster) -> Color {
    match roster.list.iter().find(|e| e.meta.uuid == owner) {
        Some(e) => e.meta.color.color(),
        None => Color::srgb(0.5, 0.5, 0.5),
    }
}

fn info_row(
    panel: &mut ChildSpawnerCommands,
    assets: &GameAssets,
    si: &ScreenInfo,
    label: &str,
    value: &str,
) {
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(sz(8.0, si)),
            ..default()
        })
        .with_children(|r| {
            r.spawn((
                Text::new(label.to_string()),
                tfont(assets, sz(11.0, si)),
                TextColor(SEC),
            ));
            r.spawn((
                Text::new(value.to_string()),
                tfont(assets, sz(11.0, si)),
                TextColor(VAL),
            ));
        });
}

fn spawn_inspector(
    commands: &mut Commands,
    assets: &GameAssets,
    roster: &Roster,
    session: &Session,
    si: &ScreenInfo,
    device: &DeviceProfile,
    meta: &TokenMeta,
    expanded: bool,
) {
    let top = sz(if device.is_mobile() { 40.0 } else { 12.0 }, si);
    let thumb = sz(if expanded { 52.0 } else { 26.0 }, si);
    let owner_txt = owner_label(meta.owner, roster, session);
    let dot = owner_color(meta.owner, roster);

    commands
        .spawn((
            InspectorRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(top),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(50),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    min_width: Val::Px(sz(200.0, si)),
                    max_width: Val::Px(sz(360.0, si)),
                    padding: UiRect::all(Val::Px(sz(8.0, si))),
                    row_gap: Val::Px(sz(6.0, si)),
                    border: UiRect::all(Val::Px(sz(1.5, si))),
                    border_radius: BorderRadius::all(Val::Px(sz(8.0, si))),
                    ..default()
                },
                BackgroundColor(BG),
                BorderColor::all(BORDER),
            ))
            .with_children(|panel| {
                // Barra (sempre): thumb + tipo + dono + handle. Toque alterna a ficha.
                panel
                    .spawn((
                        Button,
                        InspectorBar,
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(sz(8.0, si)),
                            ..default()
                        },
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            ImageNode::new(art_thumb(meta.art, assets)),
                            Node {
                                width: Val::Px(thumb),
                                height: Val::Px(thumb),
                                border_radius: BorderRadius::all(Val::Percent(50.0)),
                                ..default()
                            },
                        ));
                        bar.spawn((
                            Text::new(art_label(meta.art)),
                            tfont(assets, sz(14.0, si)),
                            TextColor(VAL),
                        ));
                        bar.spawn((
                            Node {
                                width: Val::Px(sz(10.0, si)),
                                height: Val::Px(sz(10.0, si)),
                                border_radius: BorderRadius::all(Val::Percent(50.0)),
                                ..default()
                            },
                            BackgroundColor(dot),
                        ));
                        bar.spawn((
                            Text::new(owner_txt.clone()),
                            tfont(assets, sz(12.0, si)),
                            TextColor(LBL),
                        ));
                        bar.spawn((
                            Text::new(if expanded { "  ▲" } else { "  ▼" }),
                            tfont(assets, sz(12.0, si)),
                            TextColor(SEC),
                        ));
                    });

                // Ficha (expandida): dados reais da peça.
                if expanded {
                    info_row(panel, assets, si, "Dono", &owner_txt);
                    info_row(
                        panel,
                        assets,
                        si,
                        "Posição",
                        &format!("({}, {})", meta.cell.0, meta.cell.1),
                    );
                }
            });
        });
}

type BuildKey = (Option<(u64, u64, i32, i32, u8)>, bool, u32);

fn art_code(art: TokenArt) -> u8 {
    match art {
        TokenArt::BuiltIn(i) => i,
        TokenArt::Blob(_) => 255,
    }
}

/// Reconstrói o inspector quando a seleção, os dados do token selecionado, o
/// modo expandido ou a escala mudam. Oculto quando nada está selecionado.
pub fn inspector_build(
    si: Res<ScreenInfo>,
    sel: Res<Selection>,
    state: Res<InspectorState>,
    q_tokens: Query<&Token>,
    mut last: Local<Option<BuildKey>>,
    q_root: Query<Entity, With<InspectorRoot>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    roster: Res<Roster>,
    session: Res<Session>,
    device: Res<DeviceProfile>,
) {
    let meta = sel
        .0
        .and_then(|id| q_tokens.iter().find(|t| t.meta.id == id))
        .map(|t| t.meta.clone());
    let key: BuildKey = (
        meta.as_ref()
            .map(|m| (m.id, m.owner, m.cell.0, m.cell.1, art_code(m.art))),
        state.expanded,
        (si.scale * 100.0) as u32,
    );
    if !q_root.is_empty() && *last == Some(key) {
        return;
    }
    *last = Some(key);
    for e in &q_root {
        commands.entity(e).despawn();
    }
    if let Some(m) = meta {
        spawn_inspector(
            &mut commands,
            &assets,
            &roster,
            &session,
            &si,
            &device,
            &m,
            state.expanded,
        );
    }
}

pub fn inspector_click(
    q: Query<&Interaction, (Changed<Interaction>, With<InspectorBar>)>,
    mut state: ResMut<InspectorState>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed) {
        state.expanded = !state.expanded;
    }
}

pub fn despawn_inspector(mut commands: Commands, q: Query<Entity, With<InspectorRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
