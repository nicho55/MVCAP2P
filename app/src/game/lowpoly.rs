use bevy::asset::RenderAssetUsages;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use std::collections::HashMap;

use super::terrain::Terrain;
use crate::net::Blobs;
use crate::protocol::*;
use crate::svg_assets::GameAssets;

/// Tokens e árvores são modelados para célula de 64 e escalados pelo Transform pai.
pub const BASE_CELL: f32 = 64.0;

#[derive(Resource, Clone)]
pub struct LowPoly {
    pub cube: Handle<Mesh>,
    pub hex_prism: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
    pub cone: Handle<Mesh>,
}

#[derive(Resource, Default)]
pub struct Mats {
    terrain: HashMap<(u8, i8), Handle<StandardMaterial>>,
    rings: HashMap<u8, Handle<StandardMaterial>>,
    art: HashMap<TokenArt, Handle<StandardMaterial>>,
    gold: Option<Handle<StandardMaterial>>,
    gray_ring: Option<Handle<StandardMaterial>>,
    pending: Option<Handle<StandardMaterial>>,
    trunk: Option<Handle<StandardMaterial>>,
    leaves: [Option<Handle<StandardMaterial>>; 2],
}

/// Contexto agrupado para spawn de tokens/terreno 3D (evita estourar o limite de params).
#[derive(SystemParam)]
pub struct Ctx3d<'w> {
    pub lp: Res<'w, LowPoly>,
    pub mats: ResMut<'w, Mats>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub terrain: Res<'w, Terrain>,
}

pub fn setup_lowpoly(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(LowPoly {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        hex_prism: meshes.add(hex_prism_mesh()),
        cylinder: meshes.add(Cylinder::new(1.0, 1.0)),
        cone: meshes.add(Cone {
            radius: 1.0,
            height: 1.0,
        }),
    });
}

fn flat(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.95,
        reflectance: 0.08,
        ..Default::default()
    }
}

impl Mats {
    pub fn terrain(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        assets: &GameAssets,
        tex: u8,
        elev: i8,
    ) -> Handle<StandardMaterial> {
        if let Some(h) = self.terrain.get(&(tex, elev)) {
            return h.clone();
        }
        let f = (1.0 + elev as f32 * 0.09).clamp(0.35, 1.5);
        let m = if (tex as usize) < assets.textures.len() {
            StandardMaterial {
                base_color_texture: Some(assets.textures[tex as usize].clone()),
                base_color: Color::srgb(f, f, f * 0.97),
                perceptual_roughness: 0.95,
                reflectance: 0.08,
                ..Default::default()
            }
        } else if elev >= 0 {
            flat(Color::srgb(0.72 * f, 0.68 * f, 0.58 * f))
        } else {
            flat(Color::srgb(0.20, 0.16, 0.26))
        };
        let h = materials.add(m);
        self.terrain.insert((tex, elev), h.clone());
        h
    }

    pub fn ring(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        color_idx: u8,
    ) -> Handle<StandardMaterial> {
        if let Some(h) = self.rings.get(&color_idx) {
            return h.clone();
        }
        let h = materials.add(StandardMaterial {
            base_color: palette_color(color_idx),
            perceptual_roughness: 0.55,
            reflectance: 0.25,
            ..Default::default()
        });
        self.rings.insert(color_idx, h.clone());
        h
    }

    pub fn gray_ring(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        if let Some(h) = &self.gray_ring {
            return h.clone();
        }
        let h = materials.add(flat(Color::srgb(0.55, 0.55, 0.58)));
        self.gray_ring = Some(h.clone());
        h
    }

    pub fn gold(&mut self, materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
        if let Some(h) = &self.gold {
            return h.clone();
        }
        let h = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.80, 0.22),
            emissive: LinearRgba::new(0.6, 0.45, 0.05, 1.0),
            perceptual_roughness: 0.4,
            ..Default::default()
        });
        self.gold = Some(h.clone());
        h
    }

    /// Material da arte do token; `None` se o blob ainda não chegou (usar pending + PendingArt).
    pub fn art(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        assets: &GameAssets,
        blobs: &Blobs,
        art: TokenArt,
    ) -> Option<Handle<StandardMaterial>> {
        if let Some(h) = self.art.get(&art) {
            return Some(h.clone());
        }
        let img = match art {
            TokenArt::BuiltIn(i) => {
                assets.tokens_builtin[i as usize % assets.tokens_builtin.len()].clone()
            }
            TokenArt::Blob(id) => blobs.images.get(&id)?.clone(),
        };
        let h = materials.add(StandardMaterial {
            base_color_texture: Some(img),
            perceptual_roughness: 0.8,
            reflectance: 0.1,
            ..Default::default()
        });
        self.art.insert(art, h.clone());
        Some(h)
    }

    pub fn pending(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        if let Some(h) = &self.pending {
            return h.clone();
        }
        let h = materials.add(flat(Color::srgb(0.35, 0.33, 0.40)));
        self.pending = Some(h.clone());
        h
    }

    pub fn trunk(&mut self, materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
        if let Some(h) = &self.trunk {
            return h.clone();
        }
        let h = materials.add(flat(Color::srgb(0.42, 0.29, 0.18)));
        self.trunk = Some(h.clone());
        h
    }

    pub fn leaves(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        variant: usize,
    ) -> Handle<StandardMaterial> {
        let v = variant % 2;
        if let Some(h) = &self.leaves[v] {
            return h.clone();
        }
        let c = if v == 0 {
            Color::srgb(0.25, 0.48, 0.20)
        } else {
            Color::srgb(0.33, 0.58, 0.28)
        };
        let h = materials.add(flat(c));
        self.leaves[v] = Some(h.clone());
        h
    }
}

/// Prisma hexagonal flat-top (circumraio 1, altura 1, centrado na origem),
/// vértices duplicados por face para flat shading.
fn hex_prism_mesh() -> Mesh {
    let corner = |i: usize, y: f32| -> [f32; 3] {
        let a = (60.0 * i as f32).to_radians();
        [a.cos(), y, a.sin()]
    };
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(60);
    let mut uv: Vec<[f32; 2]> = Vec::with_capacity(60);
    let cap_uv = |p: [f32; 3]| -> [f32; 2] { [p[0] * 0.5 + 0.5, p[2] * 0.5 + 0.5] };

    // topo (normal +Y)
    for i in 1..=4 {
        for p in [corner(0, 0.5), corner(i + 1, 0.5), corner(i, 0.5)] {
            uv.push(cap_uv(p));
            pos.push(p);
        }
    }
    // base (normal -Y)
    for i in 1..=4 {
        for p in [corner(0, -0.5), corner(i, -0.5), corner(i + 1, -0.5)] {
            uv.push(cap_uv(p));
            pos.push(p);
        }
    }
    // laterais
    for i in 0..6 {
        let j = (i + 1) % 6;
        let (u0, u1) = (i as f32 / 6.0, (i + 1) as f32 / 6.0);
        let ct = corner(i, 0.5);
        let cb = corner(i, -0.5);
        let jt = corner(j, 0.5);
        let jb = corner(j, -0.5);
        for (p, t) in [
            (ct, [u0, 0.0]),
            (jb, [u1, 1.0]),
            (cb, [u0, 1.0]),
            (ct, [u0, 0.0]),
            (jt, [u1, 0.0]),
            (jb, [u1, 1.0]),
        ] {
            pos.push(p);
            uv.push(t);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.compute_flat_normals();
    mesh
}

/// Árvore low-poly: tronco + dois cones de copa. Filha da entidade do mapa.
pub fn spawn_tree(
    parent: &mut ChildSpawnerCommands,
    lp: &LowPoly,
    mats: &mut Mats,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec2,
    size: f32,
    variant: usize,
) {
    let trunk_h = size * 0.5;
    let c1 = size * 1.1;
    let c2 = size * 0.8;
    parent
        .spawn((
            Transform::from_xyz(pos.x, 0.0, pos.y),
            Visibility::default(),
        ))
        .with_children(|t| {
            t.spawn((
                Mesh3d(lp.cylinder.clone()),
                MeshMaterial3d(mats.trunk(materials)),
                Transform::from_xyz(0.0, trunk_h * 0.5, 0.0).with_scale(Vec3::new(
                    size * 0.16,
                    trunk_h,
                    size * 0.16,
                )),
            ));
            t.spawn((
                Mesh3d(lp.cone.clone()),
                MeshMaterial3d(mats.leaves(materials, variant)),
                Transform::from_xyz(0.0, trunk_h + c1 * 0.5, 0.0).with_scale(Vec3::new(
                    size * 0.62,
                    c1,
                    size * 0.62,
                )),
            ));
            t.spawn((
                Mesh3d(lp.cone.clone()),
                MeshMaterial3d(mats.leaves(materials, variant + 1)),
                Transform::from_xyz(0.0, trunk_h + c1 * 0.72 + c2 * 0.5, 0.0)
                    .with_scale(Vec3::new(size * 0.45, c2, size * 0.45)),
            ));
        });
}
