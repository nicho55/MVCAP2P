use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

pub struct SvgAssetsPlugin;

impl Plugin for SvgAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_svgs);
    }
}

#[derive(Resource)]
pub struct GameAssets {
    pub textures: Vec<Handle<Image>>,
    pub tex_names: Vec<&'static str>,
    pub tokens_builtin: Vec<Handle<Image>>,
    pub logo: Handle<Image>,
    pub default_map: Handle<Image>,
    pub icons: HashMap<&'static str, Handle<Image>>,
    pub font: Option<Handle<Font>>,
}

pub fn tfont(assets: &GameAssets, size: f32) -> TextFont {
    TextFont {
        font: assets.font.clone().unwrap_or_default(),
        font_size: size,
        ..Default::default()
    }
}

fn load_svgs(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut fonts: ResMut<Assets<Font>>) {
    let t0 = std::time::Instant::now();
    let mut ras = |bytes: &[u8], px: u32| -> Handle<Image> { images.add(rasterize(bytes, px)) };

    let textures = vec![
        ras(include_bytes!("../../assets/svg/tex_grass.svg"), 256),
        ras(include_bytes!("../../assets/svg/tex_stone.svg"), 256),
        ras(include_bytes!("../../assets/svg/tex_water.svg"), 256),
        ras(include_bytes!("../../assets/svg/tex_sand.svg"), 256),
    ];
    let tokens_builtin = vec![
        ras(include_bytes!("../../assets/svg/token_warrior.svg"), 512),
        ras(include_bytes!("../../assets/svg/token_mage.svg"), 512),
        ras(include_bytes!("../../assets/svg/token_rogue.svg"), 512),
        ras(include_bytes!("../../assets/svg/token_dragon.svg"), 512),
    ];
    let logo = ras(include_bytes!("../../assets/svg/logo.svg"), 512);
    let default_map = ras(include_bytes!("../../assets/svg/map_default.svg"), 2048);

    let mut icons = HashMap::new();
    icons.insert("select", ras(include_bytes!("../../assets/svg/icon_select.svg"), 96));
    icons.insert("eraser", ras(include_bytes!("../../assets/svg/icon_eraser.svg"), 96));
    icons.insert("elev_up", ras(include_bytes!("../../assets/svg/icon_elev_up.svg"), 96));
    icons.insert("elev_down", ras(include_bytes!("../../assets/svg/icon_elev_down.svg"), 96));
    icons.insert("grid_square", ras(include_bytes!("../../assets/svg/icon_grid_square.svg"), 96));
    icons.insert("grid_hex", ras(include_bytes!("../../assets/svg/icon_grid_hex.svg"), 96));
    icons.insert("plus", ras(include_bytes!("../../assets/svg/icon_plus.svg"), 96));
    icons.insert("minus", ras(include_bytes!("../../assets/svg/icon_minus.svg"), 96));
    icons.insert("map", ras(include_bytes!("../../assets/svg/icon_map.svg"), 96));
    icons.insert("token", ras(include_bytes!("../../assets/svg/icon_token.svg"), 96));

    // Fonte com diacríticos completos (o default do bevy é um subset sem acentos).
    // Tenta o sistema primeiro; se nada existir (outra máquina, sem essas fontes),
    // cai na DejaVuSans embutida no binário — texto acentuado sempre renderiza.
    let font = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
    ]
    .iter()
    .find_map(|p| std::fs::read(p).ok())
    .and_then(|b| Font::try_from_bytes(b).ok())
    .or_else(|| Font::try_from_bytes(include_bytes!("../../assets/DejaVuSans.ttf").to_vec()).ok())
    .map(|f| fonts.add(f));
    if font.is_none() {
        warn!("nenhuma fonte carregada; acentos podem não renderizar");
    }

    info!("assets SVG rasterizados em {:?}", t0.elapsed());
    commands.insert_resource(GameAssets {
        textures,
        tex_names: vec!["Grama", "Pedra", "Água", "Areia"],
        tokens_builtin,
        logo,
        default_map,
        icons,
        font,
    });
}

fn rasterize(bytes: &[u8], target: u32) -> Image {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &opt).expect("SVG inválido");
    let sz = tree.size();
    let scale = target as f32 / sz.width().max(sz.height());
    let w = ((sz.width() * scale).round() as u32).max(1);
    let h = ((sz.height() * scale).round() as u32).max(1);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).expect("pixmap");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for p in pixmap.pixels() {
        let c = p.demultiply();
        data.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

pub fn image_from_encoded(bytes: &[u8]) -> Option<Image> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    ))
}
