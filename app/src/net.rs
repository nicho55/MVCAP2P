use bevy::prelude::*;
use bevy_matchbox::matchbox_socket::{PeerId, PeerState};
use bevy_matchbox::MatchboxSocket;
use std::collections::HashMap;

use crate::protocol::*;

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NetRx>()
            .add_message::<PeerEvent>()
            .init_resource::<Net>()
            .init_resource::<Roster>()
            .init_resource::<Blobs>()
            .add_systems(
                Update,
                (net_reconnect, net_poll, peer_greetings, blob_rx)
                    .chain()
                    .in_set(NetSet),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetSet;

#[derive(Message)]
pub struct NetRx(pub PeerId, pub Msg);

#[derive(Message)]
pub struct PeerEvent {
    pub peer: PeerId,
    pub connected: bool,
}

#[derive(Resource, Clone)]
pub struct Session {
    pub me: PlayerMeta,
    pub code: RoomCode,
    pub is_test_room: bool,
}

const MAX_RETRIES: u32 = 5;

#[derive(Resource, Default)]
pub struct Net {
    pub socket: Option<MatchboxSocket>,
    pub gm_peer: Option<PeerId>,
    pub room_url: Option<String>,
    pub reconnect: Option<Timer>,
    pub retries: u32,
}

impl Net {
    pub fn connect(&mut self, url: &str) {
        info!("conectando à sinalização: {url}");
        self.room_url = Some(url.to_string());
        self.socket = Some(MatchboxSocket::new_reliable(url));
    }

    pub fn send_to(&mut self, peer: PeerId, msg: &Msg) {
        if let Some(s) = self.socket.as_mut() {
            match bincode::serialize(msg) {
                Ok(data) => s.channel_mut(0).send(data.into_boxed_slice(), peer),
                Err(e) => warn!("falha ao serializar msg p/ envio: {e}"),
            }
        }
    }

    pub fn peers(&self) -> Vec<PeerId> {
        self.socket
            .as_ref()
            .map(|s| s.connected_peers().collect())
            .unwrap_or_default()
    }

    pub fn broadcast(&mut self, msg: &Msg) {
        for p in self.peers() {
            self.send_to(p, msg);
        }
    }

    pub fn send_gm(&mut self, msg: &Msg) {
        if let Some(p) = self.gm_peer {
            self.send_to(p, msg);
        }
    }

    pub fn disconnect(&mut self) {
        self.socket = None;
        self.gm_peer = None;
        self.room_url = None;
        self.reconnect = None;
        self.retries = 0;
    }

    /// Envia um blob em chunks; `peer = None` faz broadcast.
    pub fn send_blob_to(&mut self, peer: Option<PeerId>, id: BlobId, data: &[u8]) {
        let chunks = data.chunks(CHUNK).count() as u32;
        let start = Msg::BlobStart {
            id,
            kind: BlobKind::Image,
            len: data.len() as u32,
            chunks,
        };
        match peer {
            Some(p) => self.send_to(p, &start),
            None => self.broadcast(&start),
        }
        for (i, c) in data.chunks(CHUNK).enumerate() {
            let m = Msg::BlobChunk {
                id,
                seq: i as u32,
                data: c.to_vec(),
            };
            match peer {
                Some(p) => self.send_to(p, &m),
                None => self.broadcast(&m),
            }
        }
    }
}

#[derive(Clone)]
pub struct RosterEntry {
    pub meta: PlayerMeta,
    pub peer: Option<PeerId>,
    pub online: bool,
}

#[derive(Resource, Default)]
pub struct Roster {
    pub list: Vec<RosterEntry>,
}

impl Roster {
    pub fn upsert(&mut self, meta: PlayerMeta, peer: Option<PeerId>) {
        if let Some(e) = self.list.iter_mut().find(|e| e.meta.uuid == meta.uuid) {
            e.meta = meta;
            if peer.is_some() {
                e.peer = peer;
            }
            e.online = true;
        } else {
            self.list.push(RosterEntry {
                meta,
                peer,
                online: true,
            });
        }
    }

    pub fn by_peer(&self, p: PeerId) -> Option<&RosterEntry> {
        self.list.iter().find(|e| e.peer == Some(p))
    }

    pub fn set_peer(&mut self, uuid: PlayerUuid, peer: Option<PeerId>) {
        if let Some(e) = self.list.iter_mut().find(|e| e.meta.uuid == uuid) {
            e.peer = peer;
        }
    }

    pub fn set_online(&mut self, uuid: PlayerUuid, online: bool) {
        if let Some(e) = self.list.iter_mut().find(|e| e.meta.uuid == uuid) {
            e.online = online;
        }
    }

    pub fn set_offline_by_peer(&mut self, p: PeerId) -> Option<PlayerUuid> {
        if let Some(e) = self.list.iter_mut().find(|e| e.peer == Some(p)) {
            e.online = false;
            return Some(e.meta.uuid);
        }
        None
    }
}

struct Incoming {
    chunks: u32,
    parts: Vec<Option<Vec<u8>>>,
}

#[derive(Resource, Default)]
pub struct Blobs {
    pub data: HashMap<BlobId, Vec<u8>>,
    pub images: HashMap<BlobId, Handle<Image>>,
    incoming: HashMap<BlobId, Incoming>,
}

impl Blobs {
    pub fn store(
        &mut self,
        id: BlobId,
        bytes: Vec<u8>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        let img = crate::svg_assets::image_from_encoded(&bytes)?;
        let h = images.add(img);
        self.images.insert(id, h.clone());
        self.data.insert(id, bytes);
        Some(h)
    }
}

fn net_poll(mut net: ResMut<Net>, mut rx: MessageWriter<NetRx>, mut pev: MessageWriter<PeerEvent>) {
    if net.socket.is_none() {
        return;
    }
    let changes = match net.socket.as_mut().unwrap().try_update_peers() {
        Ok(c) => c,
        Err(e) => {
            net.retries += 1;
            if net.retries > MAX_RETRIES {
                warn!("socket caiu ({e:?}); desistindo após {MAX_RETRIES} tentativas");
                net.socket = None;
                net.gm_peer = None;
                net.reconnect = None;
                return;
            }
            warn!(
                "socket caiu ({e:?}); reconectando (tentativa {}/{MAX_RETRIES})",
                net.retries
            );
            net.socket = None;
            net.gm_peer = None;
            net.reconnect = Some(Timer::from_seconds(1.5, TimerMode::Once));
            return;
        }
    };
    for (peer, state) in changes {
        match state {
            PeerState::Connected => {
                info!("peer conectado: {peer}");
                pev.write(PeerEvent {
                    peer,
                    connected: true,
                });
            }
            PeerState::Disconnected => {
                info!("peer desconectado: {peer}");
                if net.gm_peer == Some(peer) {
                    net.gm_peer = None;
                }
                pev.write(PeerEvent {
                    peer,
                    connected: false,
                });
            }
        }
    }
    let packets = net.socket.as_mut().unwrap().channel_mut(0).receive();
    for (peer, packet) in packets {
        match bincode::deserialize::<Msg>(&packet) {
            Ok(msg) => {
                rx.write(NetRx(peer, msg));
            }
            Err(e) => warn!("pacote inválido de {peer}: {e}"),
        }
    }
}

fn peer_greetings(
    mut ev: MessageReader<PeerEvent>,
    mut net: ResMut<Net>,
    session: Option<Res<Session>>,
    mut roster: ResMut<Roster>,
) {
    let Some(sess) = session else { return };
    for e in ev.read() {
        if e.connected {
            // jogador se apresenta a todo peer novo; só o mestre responde com Welcome
            if !sess.me.is_gm {
                net.send_to(e.peer, &Msg::Hello(sess.me.clone()));
            }
        } else if let Some(uuid) = roster.set_offline_by_peer(e.peer) {
            if sess.me.is_gm {
                net.broadcast(&Msg::PlayerLeft(uuid));
            }
        }
    }
}

fn blob_rx(
    mut rx: MessageReader<NetRx>,
    mut blobs: ResMut<Blobs>,
    mut images: ResMut<Assets<Image>>,
) {
    for NetRx(_, msg) in rx.read() {
        match msg {
            Msg::BlobStart { id, chunks, .. } => {
                blobs.incoming.insert(
                    *id,
                    Incoming {
                        chunks: *chunks,
                        parts: vec![None; *chunks as usize],
                    },
                );
            }
            Msg::BlobChunk { id, seq, data } => {
                let complete = if let Some(inc) = blobs.incoming.get_mut(id) {
                    if (*seq as usize) < inc.parts.len() {
                        inc.parts[*seq as usize] = Some(data.clone());
                    }
                    inc.parts.iter().all(|p| p.is_some())
                } else {
                    false
                };
                if complete {
                    if let Some(inc) = blobs.incoming.remove(id) {
                        if let Some(bytes) = reassemble_blob(&inc.parts) {
                            info!(
                                "blob {id} completo ({} bytes, {} chunks)",
                                bytes.len(),
                                inc.chunks
                            );
                            if blobs.store(*id, bytes, &mut images).is_none() {
                                warn!("blob {id} não decodificou como imagem");
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn net_reconnect(mut net: ResMut<Net>, time: Res<Time>) {
    if net.socket.is_some() {
        return;
    }
    let finished = match net.reconnect.as_mut() {
        Some(t) => t.tick(time.delta()).is_finished(),
        None => false,
    };
    if finished {
        net.reconnect = None;
        if let Some(url) = net.room_url.clone() {
            net.connect(&url);
        }
    }
}
