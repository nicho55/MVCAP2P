use bevy::prelude::*;

use super::grid::{self, GridRes};
use super::lowpoly::Ctx3d;
use super::map::MapState;
use super::terrain::{self, ChunkRender, Terrain};
use super::tokens::{set_token_owner, spawn_token, Dragging, OwnerRing, Token};
use crate::net::{Blobs, Net, NetRx, Roster, Session};
use crate::protocol::*;
use crate::svg_assets::GameAssets;

/// GM: responde Hello com o estado completo (blobs primeiro — canal ordenado garante a chegada antes do Welcome).
pub fn handle_hello(
    mut rx: MessageReader<NetRx>,
    session: Option<Res<Session>>,
    mut net: ResMut<Net>,
    mut roster: ResMut<Roster>,
    grid: Res<GridRes>,
    terrain: Res<Terrain>,
    map_state: Res<MapState>,
    blobs: Res<Blobs>,
    q_tokens: Query<&Token>,
) {
    let Some(sess) = session else { return };
    if !sess.me.is_gm {
        return;
    }
    let hellos: Vec<_> = rx
        .read()
        .filter_map(|NetRx(p, m)| match m {
            Msg::Hello(meta) => Some((*p, meta.clone())),
            _ => None,
        })
        .collect();
    for (peer, mut meta) in hellos {
        info!("jogador entrou: {} ({peer})", meta.nick);
        meta.is_gm = false;
        roster.upsert(meta.clone(), Some(peer));
        net.broadcast(&Msg::PlayerJoined(meta));
        let mut blob_ids: Vec<BlobId> = vec![];
        if let Some(b) = map_state.want {
            blob_ids.push(b);
        }
        for t in &q_tokens {
            if let TokenArt::Blob(b) = t.meta.art {
                if !blob_ids.contains(&b) {
                    blob_ids.push(b);
                }
            }
        }
        for b in blob_ids {
            if let Some(data) = blobs.data.get(&b) {
                let data = data.clone();
                net.send_blob_to(Some(peer), b, &data);
            }
        }
        let welcome = Msg::Welcome {
            players: roster.list.iter().map(|e| e.meta.clone()).collect(),
            grid: grid.0,
            terrain: terrain.cells.iter().map(|(k, v)| (*k, *v)).collect(),
            tokens: q_tokens.iter().map(|t| t.meta.clone()).collect(),
            map_blob: map_state.want,
        };
        net.send_to(peer, &welcome);
    }
}

/// Estado global: roster, grid, mapa, terreno.
pub fn handle_core(
    mut rx: MessageReader<NetRx>,
    session: Option<Res<Session>>,
    mut net: ResMut<Net>,
    mut roster: ResMut<Roster>,
    mut grid: ResMut<GridRes>,
    mut terrain: ResMut<Terrain>,
    mut trender: ResMut<ChunkRender>,
    mut map_state: ResMut<MapState>,
) {
    let Some(sess) = session else { return };
    for NetRx(peer, msg) in rx.read() {
        match msg {
            Msg::Welcome {
                players,
                grid: g,
                terrain: t,
                map_blob,
                ..
            } if !sess.me.is_gm => {
                info!(
                    "Welcome do mestre {peer}: {} jogadores, {} células de terreno",
                    players.len(),
                    t.len()
                );
                net.gm_peer = Some(*peer);
                for p in players {
                    roster.upsert(p.clone(), None);
                }
                if let Some(gm_meta) = players.iter().find(|p| p.is_gm) {
                    roster.set_peer(gm_meta.uuid, Some(*peer));
                }
                if grid.0 != *g {
                    grid.0 = *g;
                }
                terrain.cells = t.iter().copied().collect();
                trender.full = true;
                map_state.want = *map_blob;
            }
            Msg::PlayerJoined(meta) if !sess.me.is_gm => {
                if meta.uuid != sess.me.uuid {
                    roster.upsert(meta.clone(), None);
                }
            }
            Msg::PlayerLeft(uuid) => {
                roster.set_online(*uuid, false);
            }
            Msg::Grid(g) if !sess.me.is_gm => {
                if grid.0 != *g {
                    grid.0 = *g;
                }
            }
            Msg::SetMap { blob } if !sess.me.is_gm => {
                map_state.want = *blob;
            }
            Msg::Terrain { cell, val } if !sess.me.is_gm => {
                terrain::set_cell(&mut terrain, &mut trender, *cell, *val);
            }
            _ => {}
        }
    }
}

/// Tokens: spawn/move/remove/preview, com autoridade do GM sobre requests.
/// Posições: XZ snap imediato; altura (Y) é resolvida pelo token_y_follow.
pub fn handle_tokens(
    mut commands: Commands,
    mut rx: MessageReader<NetRx>,
    session: Option<Res<Session>>,
    mut net: ResMut<Net>,
    roster: Res<Roster>,
    grid: Res<GridRes>,
    blobs: Res<Blobs>,
    assets: Res<GameAssets>,
    mut q_tokens: Query<(Entity, &mut Transform, &mut Token)>,
    mut drag: ResMut<Dragging>,
    mut ctx: Ctx3d,
) {
    let Some(sess) = session else { return };
    for NetRx(peer, msg) in rx.read() {
        match msg {
            Msg::Welcome { tokens, .. } if !sess.me.is_gm => {
                for (e, _, _) in q_tokens.iter() {
                    commands.entity(e).despawn();
                }
                drag.id = None;
                for meta in tokens {
                    spawn_token(
                        &mut commands,
                        meta.clone(),
                        &assets,
                        &blobs,
                        &grid.0,
                        &roster,
                        &mut ctx,
                    );
                }
            }
            Msg::SpawnToken(meta) => {
                if !q_tokens.iter().any(|(_, _, t)| t.meta.id == meta.id) {
                    spawn_token(
                        &mut commands,
                        meta.clone(),
                        &assets,
                        &blobs,
                        &grid.0,
                        &roster,
                        &mut ctx,
                    );
                }
            }
            Msg::SpawnTokenReq(meta) if sess.me.is_gm => {
                let Some(entry) = roster.by_peer(*peer) else {
                    continue;
                };
                let mut meta = meta.clone();
                meta.owner = entry.meta.uuid;
                if !q_tokens.iter().any(|(_, _, t)| t.meta.id == meta.id) {
                    spawn_token(
                        &mut commands,
                        meta.clone(),
                        &assets,
                        &blobs,
                        &grid.0,
                        &roster,
                        &mut ctx,
                    );
                    net.broadcast(&Msg::SpawnToken(meta));
                }
            }
            Msg::MoveTokenReq { id, cell } if sess.me.is_gm => {
                let Some(entry) = roster.by_peer(*peer) else {
                    continue;
                };
                if let Some((_, mut tf, mut tok)) =
                    q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == *id)
                {
                    if tok.meta.owner == entry.meta.uuid {
                        tok.meta.cell = *cell;
                        let c = grid::cell_center(&grid.0, *cell);
                        tf.translation.x = c.x;
                        tf.translation.z = c.y;
                        net.broadcast(&Msg::TokenMoved {
                            id: *id,
                            cell: *cell,
                        });
                    }
                }
            }
            Msg::TokenMoved { id, cell } if !sess.me.is_gm => {
                if drag.id == Some(*id) {
                    continue;
                }
                if let Some((_, mut tf, mut tok)) =
                    q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == *id)
                {
                    tok.meta.cell = *cell;
                    let c = grid::cell_center(&grid.0, *cell);
                    tf.translation.x = c.x;
                    tf.translation.z = c.y;
                }
            }
            Msg::RemoveTokenReq { id } if sess.me.is_gm => {
                let Some(entry) = roster.by_peer(*peer) else {
                    continue;
                };
                if let Some((e, _, tok)) = q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == *id) {
                    if tok.meta.owner == entry.meta.uuid {
                        commands.entity(e).despawn();
                        net.broadcast(&Msg::RemoveToken { id: *id });
                    }
                }
            }
            Msg::RemoveToken { id } => {
                if let Some((e, _, _)) = q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == *id) {
                    commands.entity(e).despawn();
                }
            }
            Msg::DragPreview { id, x, y } => {
                if drag.id == Some(*id) {
                    continue;
                }
                if let Some((_, mut tf, _)) = q_tokens.iter_mut().find(|(_, _, t)| t.meta.id == *id)
                {
                    tf.translation.x = *x;
                    tf.translation.z = *y;
                }
            }
            _ => {}
        }
    }
}

/// Processa `AssignToken` em jogadores não-mestre: atualiza dono e cor do anel.
pub fn assign_token_rx(
    mut rx: MessageReader<NetRx>,
    session: Option<Res<Session>>,
    roster: Res<Roster>,
    mut ctx: Ctx3d,
    mut q_tokens: Query<(Entity, &mut Token, &Children)>,
    mut q_rings: Query<&mut MeshMaterial3d<StandardMaterial>, With<OwnerRing>>,
) {
    let Some(sess) = session else { return };
    if sess.me.is_gm {
        return;
    }
    for NetRx(_, msg) in rx.read() {
        if let Msg::AssignToken { id, new_owner } = msg {
            set_token_owner(
                *id,
                *new_owner,
                &roster,
                &mut ctx,
                &mut q_tokens,
                &mut q_rings,
            );
        }
    }
}
