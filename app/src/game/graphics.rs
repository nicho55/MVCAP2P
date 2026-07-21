//! Opções gráficas ajustáveis em runtime — foco em desempenho em dispositivos
//! fracos (Android). Cada campo liga/desliga um custo relevante de GPU/CPU e é
//! controlável pelo painel "Gráficos" do HUD. Defaults começam baixos no Android.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

use super::camera::MainCamera;
use super::lowpoly::Vegetation;
use super::ScreenInfo;
use crate::svg_assets::{tfont, GameAssets};

const PANEL: Color = Color::srgba(0.10, 0.09, 0.14, 0.95);
const BTN_BG: Color = Color::srgb(0.16, 0.14, 0.21);
const ON: Color = Color::srgb(0.18, 0.40, 0.22);
const GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
const TEXT: Color = Color::srgb(0.92, 0.90, 0.95);

fn sz(n: f32, si: &ScreenInfo) -> f32 {
    (n * si.scale).round().max(1.0)
}

/// Nível de anti-serrilhado (MSAA). 4x é o padrão do Bevy e o mais caro.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MsaaLevel {
    Off,
    X2,
    X4,
}

impl MsaaLevel {
    fn next(self) -> Self {
        match self {
            MsaaLevel::Off => MsaaLevel::X2,
            MsaaLevel::X2 => MsaaLevel::X4,
            MsaaLevel::X4 => MsaaLevel::Off,
        }
    }
    fn label(self) -> &'static str {
        match self {
            MsaaLevel::Off => "OFF",
            MsaaLevel::X2 => "2x",
            MsaaLevel::X4 => "4x",
        }
    }
    fn to_msaa(self) -> Msaa {
        match self {
            MsaaLevel::Off => Msaa::Off,
            MsaaLevel::X2 => Msaa::Sample2,
            MsaaLevel::X4 => Msaa::Sample4,
        }
    }
}

/// Estado das opções gráficas. É a fonte única — os sistemas reagem à mudança.
#[derive(Resource, Clone)]
pub struct GraphicsSettings {
    /// Anti-serrilhado (custo de banda alto em GPUs móveis por tiles).
    pub msaa: MsaaLevel,
    /// Sombras em cascata da luz direcional (pesadas em GPU fraca).
    pub shadows: bool,
    /// Pipeline HDR + tonemapping na câmera.
    pub hdr: bool,
    /// Renderizar as árvores low-poly (draw calls).
    pub vegetation: bool,
    /// Desenhar a grade (gizmos, todo frame).
    pub grid_overlay: bool,
    /// Economia: limita a ~30 FPS (reduz calor/consumo, mantém rede fluida).
    pub power_saver: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        if cfg!(target_os = "android") {
            // Android começa enxuto para caber em aparelhos fracos.
            Self {
                msaa: MsaaLevel::Off,
                shadows: false,
                hdr: false,
                vegetation: true,
                grid_overlay: true,
                power_saver: false,
            }
        } else {
            // Desktop: qualidade cheia por padrão.
            Self {
                msaa: MsaaLevel::X4,
                shadows: true,
                hdr: true,
                vegetation: true,
                grid_overlay: true,
                power_saver: false,
            }
        }
    }
}

/// Aplica as opções ao mundo sempre que `GraphicsSettings` muda (e uma vez ao
/// iniciar, pois o recurso conta como "changed" no primeiro acesso).
pub fn apply_graphics(
    mut commands: Commands,
    settings: Res<GraphicsSettings>,
    cam: Query<Entity, With<MainCamera>>,
    mut msaa_q: Query<&mut Msaa, With<MainCamera>>,
    mut light: Query<&mut DirectionalLight>,
    mut veg: Query<&mut Visibility, With<Vegetation>>,
    mut winit: ResMut<WinitSettings>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut msaa in &mut msaa_q {
        *msaa = settings.msaa.to_msaa();
    }
    // Bevy 0.18: HDR é o componente marcador `Hdr` na câmera (não `Camera.hdr`).
    for cam_entity in &cam {
        if settings.hdr {
            commands.entity(cam_entity).insert(Hdr);
        } else {
            commands.entity(cam_entity).remove::<Hdr>();
        }
    }
    for mut l in &mut light {
        l.shadows_enabled = settings.shadows;
    }
    let vis = if settings.vegetation {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in &mut veg {
        *v = vis;
    }
    let mode = if settings.power_saver {
        UpdateMode::reactive(Duration::from_secs_f64(1.0 / 30.0))
    } else {
        UpdateMode::Continuous
    };
    winit.focused_mode = mode;
    winit.unfocused_mode = mode;
}

// ─── Painel de opções no HUD ─────────────────────────────────────────────────

#[derive(Component)]
pub struct GfxUiRoot;
#[derive(Component)]
pub struct GfxPanel;
#[derive(Component)]
pub struct GfxOpenBtn;
#[derive(Component, Clone, Copy)]
pub struct GfxToggleBtn(pub GfxOption);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GfxOption {
    Msaa,
    Shadows,
    Hdr,
    Vegetation,
    Grid,
    PowerSaver,
}

impl GfxOption {
    const ALL: [GfxOption; 6] = [
        GfxOption::Msaa,
        GfxOption::Shadows,
        GfxOption::Hdr,
        GfxOption::Vegetation,
        GfxOption::Grid,
        GfxOption::PowerSaver,
    ];
    fn name(self) -> &'static str {
        match self {
            GfxOption::Msaa => "Anti-serrilhado",
            GfxOption::Shadows => "Sombras",
            GfxOption::Hdr => "HDR",
            GfxOption::Vegetation => "Arvores",
            GfxOption::Grid => "Grade",
            GfxOption::PowerSaver => "Economia (30fps)",
        }
    }
    fn is_on(self, s: &GraphicsSettings) -> bool {
        match self {
            GfxOption::Msaa => s.msaa != MsaaLevel::Off,
            GfxOption::Shadows => s.shadows,
            GfxOption::Hdr => s.hdr,
            GfxOption::Vegetation => s.vegetation,
            GfxOption::Grid => s.grid_overlay,
            GfxOption::PowerSaver => s.power_saver,
        }
    }
    fn value(self, s: &GraphicsSettings) -> &'static str {
        match self {
            GfxOption::Msaa => s.msaa.label(),
            other => {
                if other.is_on(s) {
                    "ON"
                } else {
                    "OFF"
                }
            }
        }
    }
    fn toggle(self, s: &mut GraphicsSettings) {
        match self {
            GfxOption::Msaa => s.msaa = s.msaa.next(),
            GfxOption::Shadows => s.shadows = !s.shadows,
            GfxOption::Hdr => s.hdr = !s.hdr,
            GfxOption::Vegetation => s.vegetation = !s.vegetation,
            GfxOption::Grid => s.grid_overlay = !s.grid_overlay,
            GfxOption::PowerSaver => s.power_saver = !s.power_saver,
        }
    }
}

fn btn_text(opt: GfxOption, s: &GraphicsSettings) -> String {
    format!("{}: {}", opt.name(), opt.value(s))
}

fn toggle_btn(
    parent: &mut ChildSpawnerCommands,
    opt: GfxOption,
    s: &GraphicsSettings,
    assets: &GameAssets,
    si: &ScreenInfo,
) {
    parent
        .spawn((
            Button,
            GfxToggleBtn(opt),
            Node {
                width: Val::Px(sz(190.0, si)),
                min_height: Val::Px(sz(44.0, si)),
                padding: UiRect::all(Val::Px(sz(7.0, si))),
                margin: UiRect::all(Val::Px(sz(3.0, si))),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(if opt.is_on(s) { ON } else { BTN_BG }),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(btn_text(opt, s)),
                tfont(assets, sz(14.0, si)),
                TextColor(TEXT),
            ));
        });
}

/// Cria o botão "Gráficos" (canto superior direito) e o painel (oculto).
pub fn spawn_gfx_ui(
    mut commands: Commands,
    settings: Res<GraphicsSettings>,
    assets: Res<GameAssets>,
    si: Res<ScreenInfo>,
) {
    // Fica abaixo do botão SAIR (mesmo canto sup. direito) para não sobrepor.
    let top_clear = if cfg!(target_os = "android") {
        sz(84.0, &si)
    } else {
        sz(46.0, &si)
    };
    commands
        .spawn((
            GfxUiRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(sz(8.0, &si)),
                top: Val::Px(top_clear),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(sz(4.0, &si)),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Button,
                GfxOpenBtn,
                Node {
                    min_height: Val::Px(sz(44.0, &si)),
                    padding: UiRect::axes(Val::Px(sz(14.0, &si)), Val::Px(sz(7.0, &si))),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Graficos"),
                    tfont(&assets, sz(14.0, &si)),
                    TextColor(GOLD),
                ));
            });

            root.spawn((
                GfxPanel,
                Visibility::Hidden,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect::all(Val::Px(sz(6.0, &si))),
                    max_width: Val::Vw(70.0),
                    max_height: Val::Vh(72.0),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|panel| {
                for opt in GfxOption::ALL {
                    toggle_btn(panel, opt, &settings, &assets, &si);
                }
            });
        });
}

pub fn despawn_gfx_ui(mut commands: Commands, q: Query<Entity, With<GfxUiRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// Abre/fecha o painel ao clicar em "Gráficos".
pub fn gfx_open_click(
    q: Query<&Interaction, (Changed<Interaction>, With<GfxOpenBtn>)>,
    mut panel: Query<&mut Visibility, With<GfxPanel>>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            for mut v in &mut panel {
                *v = if *v == Visibility::Hidden {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// Aplica o toggle da opção clicada ao recurso.
pub fn gfx_toggle_click(
    q: Query<(&Interaction, &GfxToggleBtn), Changed<Interaction>>,
    mut settings: ResMut<GraphicsSettings>,
) {
    for (interaction, btn) in &q {
        if *interaction == Interaction::Pressed {
            btn.0.toggle(&mut settings);
        }
    }
}

/// Reflete o estado atual nos rótulos/cores dos botões do painel.
pub fn gfx_panel_visuals(
    settings: Res<GraphicsSettings>,
    mut q_btn: Query<(&GfxToggleBtn, &Children, &mut BackgroundColor)>,
    mut q_text: Query<&mut Text>,
) {
    if !settings.is_changed() {
        return;
    }
    for (btn, children, mut bg) in &mut q_btn {
        *bg = BackgroundColor(if btn.0.is_on(&settings) { ON } else { BTN_BG });
        for child in children {
            if let Ok(mut text) = q_text.get_mut(*child) {
                *text = Text::new(btn_text(btn.0, &settings));
            }
        }
    }
}
