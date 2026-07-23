use bevy::asset::RenderAssetUsages;
use bevy::input::touch::TouchPhase;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::camera::{cursor_ground, CamRig, MainCamera};
use super::grid::{self, GridRes};
use super::lowpoly::Ctx3d;
use super::tokens::TouchDrag;
use super::{ActiveTool, UiHovered};
use crate::net::{Net, Session};
use crate::protocol::*;
use crate::svg_assets::GameAssets;

pub const CHUNK_SIZE: i32 = 8;

pub type ChunkCoord = (i32, i32);

pub fn cell_to_chunk(cell: Cell) -> ChunkCoord {
    (cell.0 >> 3, cell.1 >> 3)
}

#[derive(Resource, Default)]
pub struct Terrain {
    pub cells: HashMap<Cell, TerrainCell>,
}

#[derive(Resource)]
pub struct ChunkRender {
    pub meshes: HashMap<ChunkCoord, Entity>,
    pub dirty: HashSet<ChunkCoord>,
    pub full: bool,
    pub active_radius: u32,
}

impl Default for ChunkRender {
    fn default() -> Self {
        Self {
            meshes: HashMap::new(),
            dirty: HashSet::new(),
            full: false,
            active_radius: 6,
        }
    }
}

pub fn elev_height(cell: f32, elev: i8) -> f32 {
    let tile = cell * 0.10;
    if elev <= 0 {
        tile
    } else {
        tile + elev as f32 * cell * 0.28
    }
}

pub fn cell_top(terrain: &Terrain, g: &GridCfg, cell: Cell) -> f32 {
    terrain
        .cells
        .get(&cell)
        .map(|v| elev_height(g.cell, v.elev))
        .unwrap_or(0.0)
}

pub fn set_cell(
    terrain: &mut Terrain,
    render: &mut ChunkRender,
    cell: Cell,
    val: Option<TerrainCell>,
) -> bool {
    let changed = match val {
        Some(v) => terrain.cells.get(&cell) != Some(&v),
        None => terrain.cells.contains_key(&cell),
    };
    if changed {
        match val {
            Some(v) => {
                terrain.cells.insert(cell, v);
            }
            None => {
                terrain.cells.remove(&cell);
            }
        }
        render.dirty.insert(cell_to_chunk(cell));
    }
    changed
}

enum Op {
    Paint(u8),
    Erase,
    Elev(i8),
}

pub fn terrain_tool(
    buttons: Res<ButtonInput<MouseButton>>,
    mut touch_ev: MessageReader<TouchInput>,
    tool: Res<ActiveTool>,
    ui: Res<UiHovered>,
    drag: Res<TouchDrag>,
    session: Res<Session>,
    windows: Query<&Window>,
    q_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid: Res<GridRes>,
    mut terrain: ResMut<Terrain>,
    mut render: ResMut<ChunkRender>,
    mut net: ResMut<Net>,
    mut stroke: Local<HashSet<Cell>>,
) {
    if !session.me.is_gm {
        return;
    }
    let op = match *tool {
        ActiveTool::Paint(i) => Op::Paint(i),
        ActiveTool::Erase => Op::Erase,
        ActiveTool::ElevUp => Op::Elev(1),
        ActiveTool::ElevDown => Op::Elev(-1),
        ActiveTool::Select => return,
    };
    if buttons.just_released(MouseButton::Left) {
        stroke.clear();
    }
    let mut touch_ended = false;
    let mut touch_active = false;
    for t in touch_ev.read() {
        match t.phase {
            TouchPhase::Started | TouchPhase::Moved => touch_active = true,
            TouchPhase::Ended | TouchPhase::Canceled => touch_ended = true,
        }
    }
    if touch_ended {
        stroke.clear();
    }
    if drag.token_id.is_some() {
        return;
    }
    let mouse_down = buttons.pressed(MouseButton::Left);
    if !mouse_down && !touch_active {
        return;
    }
    if ui.0 && (mouse_down || touch_active) {
        return;
    }
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_gt)) = q_cam.single() else {
        return;
    };
    let Some(world) = cursor_ground(win, cam, cam_gt) else {
        return;
    };
    let cell = grid::world_to_cell(&grid.0, world);
    if stroke.contains(&cell) {
        return;
    }
    stroke.insert(cell);
    let old = terrain.cells.get(&cell).copied();
    let val: Option<TerrainCell> = match op {
        Op::Paint(i) => Some(TerrainCell {
            tex: i,
            elev: old.map(|o| o.elev).unwrap_or(0),
        }),
        Op::Erase => None,
        Op::Elev(d) => {
            let o = old.unwrap_or(TerrainCell {
                tex: TEX_NONE,
                elev: 0,
            });
            Some(TerrainCell {
                tex: o.tex,
                elev: (o.elev + d).clamp(-4, 4),
            })
        }
    };
    if set_cell(&mut terrain, &mut render, cell, val) {
        net.broadcast(&Msg::Terrain { cell, val });
    }
}

/// Marca o chunk como componente ECS para fácil query/despawn.
#[derive(Component)]
struct ChunkMarker;

/// Calcula quais chunks estão dentro do raio de visão da câmera.
fn visible_chunks(rig: &CamRig, grid: &GridCfg, radius: u32) -> HashSet<ChunkCoord> {
    let focus_cell = grid::world_to_cell(grid, Vec2::new(rig.focus.x, rig.focus.z));
    let center_chunk = cell_to_chunk(focus_cell);
    let r = radius as i32;
    let mut visible = HashSet::new();
    for dx in -r..=r {
        for dy in -r..=r {
            visible.insert((center_chunk.0 + dx, center_chunk.1 + dy));
        }
    }
    visible
}

/// Substitui terrain_render(). Gera uma mesh combinada por chunk,
/// despawna chunks fora do raio de visão.
pub fn chunk_render_system(
    mut commands: Commands,
    mut render: ResMut<ChunkRender>,
    grid: Res<GridRes>,
    rig: Res<CamRig>,
    assets: Res<GameAssets>,
    mut ctx: Ctx3d,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    if render.full {
        render.full = false;
        let ents: Vec<(ChunkCoord, Entity)> = render.meshes.drain().collect();
        for (_, e) in ents {
            commands.entity(e).despawn();
        }
        let chunks: HashSet<ChunkCoord> = ctx
            .terrain
            .cells
            .keys()
            .map(|c| cell_to_chunk(*c))
            .collect();
        render.dirty.extend(chunks);
    }

    let visible = visible_chunks(&rig, &grid.0, render.active_radius);

    // Despawnar chunks que saíram do raio
    let outside: Vec<ChunkCoord> = render
        .meshes
        .keys()
        .filter(|cc| !visible.contains(cc))
        .copied()
        .collect();
    for cc in outside {
        if let Some(e) = render.meshes.remove(&cc) {
            commands.entity(e).despawn();
        }
    }

    // Marcar chunks que entraram no raio como dirty (se têm células)
    for &cc in &visible {
        if !render.meshes.contains_key(&cc) && chunk_has_cells(&ctx.terrain, cc) {
            render.dirty.insert(cc);
        }
    }

    if render.dirty.is_empty() {
        return;
    }

    let dirty = std::mem::take(&mut render.dirty);
    for cc in dirty {
        if !visible.contains(&cc) {
            continue;
        }

        // Despawnar entity antiga do chunk
        if let Some(e) = render.meshes.remove(&cc) {
            commands.entity(e).despawn();
        }

        // Coletar células deste chunk
        let cells = collect_chunk_cells(&ctx.terrain, cc);
        if cells.is_empty() {
            continue;
        }

        // Escolher material: usar o tex/elev mais comum do chunk
        let (tex, elev) = dominant_terrain(&cells);
        let mat = ctx.mats.terrain(&mut ctx.materials, &assets, tex, elev);

        // Gerar mesh combinada
        let mesh = build_chunk_mesh(&cells, &grid.0, cc);
        let mesh_handle = mesh_assets.add(mesh);

        let e = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(mat),
                Transform::IDENTITY,
                ChunkMarker,
            ))
            .id();
        render.meshes.insert(cc, e);
    }
}

fn chunk_has_cells(terrain: &Terrain, cc: ChunkCoord) -> bool {
    let bx = cc.0 * CHUNK_SIZE;
    let by = cc.1 * CHUNK_SIZE;
    for lx in 0..CHUNK_SIZE {
        for ly in 0..CHUNK_SIZE {
            if terrain.cells.contains_key(&(bx + lx, by + ly)) {
                return true;
            }
        }
    }
    false
}

fn collect_chunk_cells(terrain: &Terrain, cc: ChunkCoord) -> Vec<(Cell, TerrainCell)> {
    let bx = cc.0 * CHUNK_SIZE;
    let by = cc.1 * CHUNK_SIZE;
    let mut cells = Vec::new();
    for lx in 0..CHUNK_SIZE {
        for ly in 0..CHUNK_SIZE {
            let cell = (bx + lx, by + ly);
            if let Some(tc) = terrain.cells.get(&cell) {
                cells.push((cell, *tc));
            }
        }
    }
    cells
}

fn dominant_terrain(cells: &[(Cell, TerrainCell)]) -> (u8, i8) {
    let mut counts: HashMap<(u8, i8), usize> = HashMap::new();
    for (_, tc) in cells {
        *counts.entry((tc.tex, tc.elev)).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(k, _)| k)
        .unwrap_or((TEX_NONE, 0))
}

/// Gera uma mesh combinada com todos os prismas do chunk.
/// Cada célula vira um prisma (cube ou hex) posicionado em world-space.
fn build_chunk_mesh(cells: &[(Cell, TerrainCell)], grid: &GridCfg, _cc: ChunkCoord) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for &(cell, tc) in cells {
        let center = grid::cell_center(grid, cell);
        let h = elev_height(grid.cell, tc.elev);

        match grid.kind {
            GridKind::Square => {
                let half = grid.cell * 0.99 * 0.5;
                let y_top = h + 0.02;
                let y_bot = 0.02;
                let base_idx = positions.len() as u32;
                append_cube(
                    &mut positions,
                    &mut uvs,
                    &mut indices,
                    center,
                    half,
                    y_bot,
                    y_top,
                    base_idx,
                );
            }
            GridKind::HexFlat => {
                let s = grid.cell * 0.5 * 0.995;
                let y_top = h + 0.02;
                let y_bot = 0.02;
                append_hex_prism(
                    &mut positions,
                    &mut uvs,
                    &mut indices,
                    center,
                    s,
                    y_bot,
                    y_top,
                );
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices));
    mesh.compute_flat_normals();
    mesh
}

fn append_cube(
    pos: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec2,
    half: f32,
    y_bot: f32,
    y_top: f32,
    base: u32,
) {
    let cx = center.x;
    let cz = center.y;
    let (l, r, f, b) = (cx - half, cx + half, cz - half, cz + half);

    // 6 faces × 4 vertices = 24 vertices
    #[rustfmt::skip]
    let verts: [[f32; 3]; 24] = [
        // top (+Y)
        [l, y_top, f], [r, y_top, f], [r, y_top, b], [l, y_top, b],
        // bottom (-Y)
        [l, y_bot, b], [r, y_bot, b], [r, y_bot, f], [l, y_bot, f],
        // front (-Z)
        [l, y_bot, f], [r, y_bot, f], [r, y_top, f], [l, y_top, f],
        // back (+Z)
        [r, y_bot, b], [l, y_bot, b], [l, y_top, b], [r, y_top, b],
        // left (-X)
        [l, y_bot, b], [l, y_bot, f], [l, y_top, f], [l, y_top, b],
        // right (+X)
        [r, y_bot, f], [r, y_bot, b], [r, y_top, b], [r, y_top, f],
    ];

    #[rustfmt::skip]
    let face_uvs: [[f32; 2]; 4] = [
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
    ];

    for v in &verts {
        pos.push(*v);
    }
    for _ in 0..6 {
        for uv in &face_uvs {
            uvs.push(*uv);
        }
    }

    for face in 0..6u32 {
        let fb = base + face * 4;
        indices.extend_from_slice(&[fb, fb + 1, fb + 2, fb, fb + 2, fb + 3]);
    }
}

fn append_hex_prism(
    pos: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec2,
    circumradius: f32,
    y_bot: f32,
    y_top: f32,
) {
    let cx = center.x;
    let cz = center.y;

    let corner = |i: usize, y: f32| -> [f32; 3] {
        let a = (60.0 * i as f32).to_radians();
        [cx + a.cos() * circumradius, y, cz + a.sin() * circumradius]
    };
    let cap_uv = |p: [f32; 3]| -> [f32; 2] {
        [
            (p[0] - cx) / circumradius * 0.5 + 0.5,
            (p[2] - cz) / circumradius * 0.5 + 0.5,
        ]
    };

    // Top cap: 4 triangles (fan from corner 0)
    for i in 1..=4u32 {
        let p0 = corner(0, y_top);
        let p1 = corner((i + 1) as usize, y_top);
        let p2 = corner(i as usize, y_top);
        let b = pos.len() as u32;
        for p in [p0, p1, p2] {
            uvs.push(cap_uv(p));
            pos.push(p);
        }
        indices.extend_from_slice(&[b, b + 1, b + 2]);
    }

    // Bottom cap: 4 triangles
    for i in 1..=4u32 {
        let p0 = corner(0, y_bot);
        let p1 = corner(i as usize, y_bot);
        let p2 = corner((i + 1) as usize, y_bot);
        let b = pos.len() as u32;
        for p in [p0, p1, p2] {
            uvs.push(cap_uv(p));
            pos.push(p);
        }
        indices.extend_from_slice(&[b, b + 1, b + 2]);
    }

    // 6 side faces (2 triangles each)
    for i in 0..6usize {
        let j = (i + 1) % 6;
        let u0 = i as f32 / 6.0;
        let u1 = (i + 1) as f32 / 6.0;
        let ct = corner(i, y_top);
        let cb = corner(i, y_bot);
        let jt = corner(j, y_top);
        let jb = corner(j, y_bot);

        let b = pos.len() as u32;
        pos.extend_from_slice(&[ct, jb, cb, ct, jt, jb]);
        uvs.extend_from_slice(&[
            [u0, 0.0],
            [u1, 1.0],
            [u0, 1.0],
            [u0, 0.0],
            [u1, 0.0],
            [u1, 1.0],
        ]);
        indices.extend_from_slice(&[b, b + 1, b + 2, b + 3, b + 4, b + 5]);
    }
}
