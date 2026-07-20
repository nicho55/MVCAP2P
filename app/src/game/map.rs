use bevy::prelude::*;
use bevy::window::FileDragAndDrop;

use super::camera::{cursor_ground, MainCamera};
use super::grid::{self, GridRes};
use super::lowpoly::{self, Ctx3d};
use super::tokens;
use crate::net::{Blobs, Net, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::GameAssets;

#[derive(Resource, Default)]
pub struct MapState {
    /// None = mapa padrão embutido; Some(blob) = imagem importada.
    pub want: Option<BlobId>,
    pub applied: Option<Option<BlobId>>,
    pub size: Vec2,
}

#[derive(Resource, Default, PartialEq, Clone, Copy)]
pub enum DropMode {
    #[default]
    Token,
    Map,
}

#[derive(Component)]
pub struct MapGround;

/// Aplica o mapa desejado: plano texturizado no chão (XZ), centrado na origem.
/// No mapa padrão, decora com árvores low-poly (filhas do plano — somem juntas).
pub fn sync_map(
    mut commands: Commands,
    mut map_state: ResMut<MapState>,
    blobs: Res<Blobs>,
    assets: Res<GameAssets>,
    images: Res<Assets<Image>>,
    q_map: Query<Entity, With<MapGround>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ctx: Ctx3d,
) {
    if map_state.applied == Some(map_state.want) {
        return;
    }
    let handle = match map_state.want {
        None => assets.default_map.clone(),
        Some(id) => match blobs.images.get(&id) {
            Some(h) => h.clone(),
            None => return,
        },
    };
    let Some(img) = images.get(&handle) else {
        return;
    };
    let size = img.size_f32();
    for e in &q_map {
        commands.entity(e).despawn();
    }
    let plane = meshes.add(Plane3d::default().mesh().size(size.x, size.y));
    let mat = ctx.materials.add(StandardMaterial {
        base_color_texture: Some(handle),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.02,
        ..Default::default()
    });
    let is_default = map_state.want.is_none();
    let lp = (*ctx.lp).clone();
    commands
        .spawn((
            Mesh3d(plane),
            MeshMaterial3d(mat),
            Transform::IDENTITY,
            MapGround,
            Visibility::default(),
        ))
        .with_children(|p| {
            if is_default {
                // bosques do mapa padrão (posições casadas com as manchas verdes do SVG)
                let groves = [
                    (Vec2::new(-790.0, 660.0), 90.0, 0),
                    (Vec2::new(-710.0, 610.0), 66.0, 1),
                    (Vec2::new(-850.0, 730.0), 58.0, 1),
                    (Vec2::new(780.0, -740.0), 84.0, 0),
                    (Vec2::new(722.0, -690.0), 56.0, 1),
                    (Vec2::new(258.0, -172.0), 62.0, 0),
                    (Vec2::new(318.0, -128.0), 46.0, 1),
                    (Vec2::new(902.0, 830.0), 70.0, 0),
                ];
                for (pos, sz, v) in groves {
                    lowpoly::spawn_tree(p, &lp, &mut ctx.mats, &mut ctx.materials, pos, sz, v);
                }
            }
        });
    info!("mapa aplicado ({}x{})", size.x, size.y);
    map_state.size = size;
    let want = map_state.want;
    map_state.applied = Some(want);
}

pub fn import_map_bytes(
    bytes: Vec<u8>,
    blobs: &mut Blobs,
    images: &mut Assets<Image>,
    net: &mut Net,
    map_state: &mut MapState,
) {
    let id: BlobId = rand::random();
    if blobs.store(id, bytes.clone(), images).is_none() {
        warn!("imagem de mapa inválida");
        return;
    }
    net.send_blob_to(None, id, &bytes);
    net.broadcast(&Msg::SetMap { blob: Some(id) });
    map_state.want = Some(id);
}

pub fn file_drop(
    mut evr: MessageReader<FileDragAndDrop>,
    drop_mode: Res<DropMode>,
    session: Res<Session>,
    mut net: ResMut<Net>,
    mut blobs: ResMut<Blobs>,
    mut images: ResMut<Assets<Image>>,
    mut map_state: ResMut<MapState>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    grid: Res<GridRes>,
    roster: Res<Roster>,
    windows: Query<&Window>,
    q_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ctx: Ctx3d,
) {
    for ev in evr.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = ev else {
            continue;
        };
        let ext = path_buf
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp") {
            warn!("formato não suportado: .{ext} (use PNG, JPEG ou WebP)");
            continue;
        }
        let bytes = match std::fs::read(path_buf) {
            Ok(b) => b,
            Err(e) => {
                warn!("falha lendo {}: {e}", path_buf.display());
                continue;
            }
        };
        match *drop_mode {
            DropMode::Map => {
                if !session.me.is_gm {
                    warn!("apenas o mestre pode trocar o mapa");
                    continue;
                }
                info!("importando mapa: {}", path_buf.display());
                import_map_bytes(bytes, &mut blobs, &mut images, &mut net, &mut map_state);
            }
            DropMode::Token => {
                let blob_id: BlobId = rand::random();
                if blobs.store(blob_id, bytes.clone(), &mut images).is_none() {
                    warn!("imagem inválida: {}", path_buf.display());
                    continue;
                }
                net.send_blob_to(None, blob_id, &bytes);
                let cell = windows
                    .single()
                    .ok()
                    .zip(q_cam.single().ok())
                    .and_then(|(w, (c, gt))| cursor_ground(w, c, gt))
                    .map(|wp| grid::world_to_cell(&grid.0, wp))
                    .unwrap_or((0, 0));
                let meta = TokenMeta {
                    id: rand::random(),
                    owner: session.me.uuid,
                    art: TokenArt::Blob(blob_id),
                    cell,
                };
                info!("criando token de {}", path_buf.display());
                if session.me.is_gm {
                    tokens::spawn_token(
                        &mut commands,
                        meta.clone(),
                        &assets,
                        &blobs,
                        &grid.0,
                        &roster,
                        &mut ctx,
                    );
                    net.broadcast(&Msg::SpawnToken(meta));
                } else {
                    net.send_gm(&Msg::SpawnTokenReq(meta));
                }
            }
        }
    }
}
