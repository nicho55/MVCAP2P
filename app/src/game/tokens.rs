use bevy::prelude::*;

use super::camera::{cursor_ground, cursor_ray, ray_point_dist, MainCamera};
use super::grid::{self, GridRes};
use super::lowpoly::{Ctx3d, BASE_CELL};
use super::terrain::{cell_top, Terrain};
use super::{ActiveTool, UiHovered};
use crate::net::{Blobs, Net, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::GameAssets;

#[derive(Component)]
pub struct Token {
    pub meta: TokenMeta,
}

#[derive(Component)]
pub struct PendingArt(pub BlobId);

#[derive(Component)]
pub struct OwnerRing;

#[derive(Component)]
pub struct ArtDisc;

#[derive(Component)]
pub struct SelRing;

#[derive(Resource, Default)]
pub struct Selection(pub Option<TokenId>);

#[derive(Resource, Default)]
pub struct Dragging {
    pub id: Option<TokenId>,
    pub grab: Vec2,
    pub last_tx: f32,
}

pub fn token_size(g: &GridCfg) -> f32 {
    match g.kind {
        GridKind::Square => g.cell * 0.86,
        GridKind::HexFlat => g.cell * 0.78,
    }
}

/// Peça de tabuleiro 3D: anel da cor do dono + disco com a arte em cima.
/// Filhos modelados para célula BASE_CELL; o pai escala por g.cell/BASE_CELL.
pub fn spawn_token(
    commands: &mut Commands,
    meta: TokenMeta,
    assets: &GameAssets,
    blobs: &Blobs,
    g: &GridCfg,
    roster: &Roster,
    ctx: &mut Ctx3d,
) {
    let s = g.cell / BASE_CELL;
    let r = token_size(&GridCfg { kind: g.kind, cell: BASE_CELL }) * 0.5;
    let c = grid::cell_center(g, meta.cell);
    let top = cell_top(&ctx.terrain, g, meta.cell);

    let ring_mat = match roster.list.iter().find(|e| e.meta.uuid == meta.owner) {
        Some(e) => ctx.mats.ring(&mut ctx.materials, e.meta.color),
        None => ctx.mats.gray_ring(&mut ctx.materials),
    };
    let (art_mat, pending) = match ctx.mats.art(&mut ctx.materials, assets, blobs, meta.art) {
        Some(h) => (h, None),
        None => {
            let pb = match meta.art {
                TokenArt::Blob(id) => Some(id),
                _ => None,
            };
            (ctx.mats.pending(&mut ctx.materials), pb)
        }
    };
    let gold = ctx.mats.gold(&mut ctx.materials);

    let mut ec = commands.spawn((
        Transform::from_xyz(c.x, top, c.y).with_scale(Vec3::splat(s)),
        Visibility::default(),
        Token { meta },
    ));
    ec.with_children(|p| {
        p.spawn((
            Mesh3d(ctx.lp.cylinder.clone()),
            MeshMaterial3d(ring_mat),
            Transform::from_xyz(0.0, 3.5, 0.0).with_scale(Vec3::new(r * 1.18, 7.0, r * 1.18)),
            OwnerRing,
        ));
        p.spawn((
            Mesh3d(ctx.lp.cylinder.clone()),
            MeshMaterial3d(art_mat),
            Transform::from_xyz(0.0, 7.0 + 5.0, 0.0).with_scale(Vec3::new(r, 10.0, r)),
            ArtDisc,
        ));
        p.spawn((
            Mesh3d(ctx.lp.cylinder.clone()),
            MeshMaterial3d(gold),
            Transform::from_xyz(0.0, 1.2, 0.0).with_scale(Vec3::new(r * 1.5, 2.4, r * 1.5)),
            SelRing,
            Visibility::Hidden,
        ));
    });
    if let Some(b) = pending {
        ec.insert(PendingArt(b));
    }
}

pub fn token_interact(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    q_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_tokens: Query<(Entity, &mut Transform, &mut Token)>,
    session: Res<Session>,
    tool: Res<ActiveTool>,
    ui: Res<UiHovered>,
    mut sel: ResMut<Selection>,
    mut drag: ResMut<Dragging>,
    mut net: ResMut<Net>,
    grid: Res<GridRes>,
    terrain: Res<Terrain>,
    time: Res<Time>,
) {
    if *tool != ActiveTool::Select {
        return;
    }
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_gt)) = q_cam.single() else { return };

    if buttons.just_pressed(MouseButton::Left) && !ui.0 {
        if let Some(ray) = cursor_ray(win, cam, cam_gt) {
            let radius = token_size(&grid.0) * 0.62;
            let mut best: Option<(TokenId, PlayerUuid, f32)> = None;
            for (_, tf, tok) in q_tokens.iter() {
                // centro da peça (um pouco acima da base)
                let p = tf.translation + Vec3::Y * grid.0.cell * 0.15;
                let d = ray_point_dist(&ray, p);
                if d <= radius && best.map(|(_, _, bd)| d < bd).unwrap_or(true) {
                    best = Some((tok.meta.id, tok.meta.owner, d));
                }
            }
            if let Some((id, owner, _)) = best {
                sel.0 = Some(id);
                if session.me.is_gm || owner == session.me.uuid {
                    if let Some(ground) = cursor_ground(win, cam, cam_gt) {
                        let tokxz = q_tokens
                            .iter()
                            .find(|(_, _, t)| t.meta.id == id)
                            .map(|(_, tf, _)| Vec2::new(tf.translation.x, tf.translation.z))
                            .unwrap_or(ground);
                        drag.id = Some(id);
                        drag.grab = tokxz - ground;
                        drag.last_tx = 0.0;
                    }
                }
            } else {
                sel.0 = None;
            }
        }
    }

    let Some(id) = drag.id else { return };

    if buttons.pressed(MouseButton::Left) {
        if let Some(ground) = cursor_ground(win, cam, cam_gt) {
            let pos = ground + drag.grab;
            let hover_cell = grid::world_to_cell(&grid.0, pos);
            let lift = cell_top(&terrain, &grid.0, hover_cell) + grid.0.cell * 0.35;
            if let Some((_, mut tf, _)) = q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == id) {
                tf.translation.x = pos.x;
                tf.translation.z = pos.y;
                tf.translation.y = lift;
            }
            let now = time.elapsed_secs();
            if now - drag.last_tx > 0.05 {
                drag.last_tx = now;
                net.broadcast(&Msg::DragPreview { id, x: pos.x, y: pos.y });
            }
        }
    }

    if buttons.just_released(MouseButton::Left) {
        drag.id = None;
        if let Some((_, mut tf, mut tok)) = q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == id) {
            let pos = cursor_ground(win, cam, cam_gt)
                .map(|g| g + drag.grab)
                .unwrap_or(Vec2::new(tf.translation.x, tf.translation.z));
            let cell = grid::world_to_cell(&grid.0, pos);
            tok.meta.cell = cell;
            let c = grid::cell_center(&grid.0, cell);
            tf.translation.x = c.x;
            tf.translation.z = c.y;
            if session.me.is_gm {
                net.broadcast(&Msg::TokenMoved { id, cell });
            } else {
                net.send_gm(&Msg::MoveTokenReq { id, cell });
            }
        }
    }
}

/// A peça acompanha suavemente a altura do terreno da sua célula
/// (sobe/desce quando o GM deforma o chão embaixo dela).
pub fn token_y_follow(
    time: Res<Time>,
    terrain: Res<Terrain>,
    grid: Res<GridRes>,
    drag: Res<Dragging>,
    mut q: Query<(&mut Transform, &Token)>,
) {
    let k = (time.delta_secs() * 10.0).min(1.0);
    for (mut tf, tok) in &mut q {
        if drag.id == Some(tok.meta.id) {
            continue;
        }
        let target = cell_top(&terrain, &grid.0, tok.meta.cell);
        let cur = tf.translation.y;
        let next = cur + (target - cur) * k;
        if (next - cur).abs() > 0.01 {
            tf.translation.y = next;
        }
    }
}

pub fn selection_visual(
    sel: Res<Selection>,
    q_tokens: Query<(&Token, &Children)>,
    mut q_rings: Query<&mut Visibility, With<SelRing>>,
) {
    for (tok, children) in &q_tokens {
        let vis = if sel.0 == Some(tok.meta.id) { Visibility::Inherited } else { Visibility::Hidden };
        for c in children {
            if let Ok(mut v) = q_rings.get_mut(*c) {
                if *v != vis {
                    *v = vis;
                }
            }
        }
    }
}

pub fn delete_selected(
    keys: Res<ButtonInput<KeyCode>>,
    mut sel: ResMut<Selection>,
    session: Res<Session>,
    mut net: ResMut<Net>,
    mut commands: Commands,
    q_tokens: Query<(Entity, &Token)>,
) {
    if !keys.just_pressed(KeyCode::Delete) {
        return;
    }
    let Some(id) = sel.0 else { return };
    let Some((e, tok)) = q_tokens.iter().find(|(_, t)| t.meta.id == id) else { return };
    if session.me.is_gm {
        commands.entity(e).despawn();
        net.broadcast(&Msg::RemoveToken { id });
        sel.0 = None;
    } else if tok.meta.owner == session.me.uuid {
        commands.entity(e).despawn();
        net.send_gm(&Msg::RemoveTokenReq { id });
        sel.0 = None;
    }
}

/// Troca o material do disco de arte quando o blob da imagem chega.
pub fn resolve_pending_art(
    mut commands: Commands,
    blobs: Res<Blobs>,
    assets: Res<GameAssets>,
    mut ctx: Ctx3d,
    q_pending: Query<(Entity, &PendingArt, &Children)>,
    mut q_art: Query<&mut MeshMaterial3d<StandardMaterial>, With<ArtDisc>>,
) {
    for (e, p, children) in &q_pending {
        if let Some(h) = ctx.mats.art(&mut ctx.materials, &assets, &blobs, TokenArt::Blob(p.0)) {
            for c in children {
                if let Ok(mut m) = q_art.get_mut(*c) {
                    m.0 = h.clone();
                }
            }
            commands.entity(e).remove::<PendingArt>();
        }
    }
}

/// Atualiza a cor do anel quando o roster muda (dono entrou/trocou de cor).
pub fn refresh_ring_colors(
    roster: Res<Roster>,
    mut ctx: Ctx3d,
    q_tokens: Query<(&Token, &Children)>,
    mut q_rings: Query<&mut MeshMaterial3d<StandardMaterial>, With<OwnerRing>>,
) {
    if !roster.is_changed() {
        return;
    }
    for (tok, children) in &q_tokens {
        let mat = match roster.list.iter().find(|e| e.meta.uuid == tok.meta.owner) {
            Some(e) => ctx.mats.ring(&mut ctx.materials, e.meta.color),
            None => ctx.mats.gray_ring(&mut ctx.materials),
        };
        for c in children {
            if let Ok(mut m) = q_rings.get_mut(*c) {
                if m.0 != mat {
                    m.0 = mat.clone();
                }
            }
        }
    }
}
