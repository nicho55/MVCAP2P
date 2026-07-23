use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::camera::{cursor_ground, MainCamera};
use super::grid::{self, GridRes};
use super::lowpoly::Ctx3d;
use super::tokens::TouchDrag;
use super::{ActiveTool, UiHovered};
use crate::net::{Net, Session};
use crate::protocol::*;
use crate::svg_assets::GameAssets;

#[derive(Resource, Default)]
pub struct Terrain {
    pub cells: HashMap<Cell, TerrainCell>,
}

#[derive(Resource, Default)]
pub struct TerrainRender {
    pub ents: HashMap<Cell, Entity>,
    pub dirty: Vec<Cell>,
    pub full: bool,
}

/// Altura da coluna de uma célula. Elevação <= 0 vira um ladrilho fino
/// (rebaixo é indicado por cor escura — não dá para cavar abaixo do plano do mapa).
pub fn elev_height(cell: f32, elev: i8) -> f32 {
    let tile = cell * 0.10;
    if elev <= 0 {
        tile
    } else {
        tile + elev as f32 * cell * 0.28
    }
}

/// Altura do topo da célula (onde uma peça deve apoiar).
pub fn cell_top(terrain: &Terrain, g: &GridCfg, cell: Cell) -> f32 {
    terrain
        .cells
        .get(&cell)
        .map(|v| elev_height(g.cell, v.elev))
        .unwrap_or(0.0)
}

pub fn set_cell(
    terrain: &mut Terrain,
    render: &mut TerrainRender,
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
        render.dirty.push(cell);
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
    mut render: ResMut<TerrainRender>,
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

/// Materializa células como prismas low-poly (cubo no grid quadrado,
/// prisma hexagonal no grid hex), com altura = elevação.
pub fn terrain_render(
    mut commands: Commands,
    mut render: ResMut<TerrainRender>,
    grid: Res<GridRes>,
    assets: Res<GameAssets>,
    mut ctx: Ctx3d,
) {
    if render.full {
        render.full = false;
        let ents: Vec<Entity> = render.ents.drain().map(|(_, e)| e).collect();
        for e in ents {
            commands.entity(e).despawn();
        }
        render.dirty = ctx.terrain.cells.keys().copied().collect();
    }
    if render.dirty.is_empty() {
        return;
    }
    let dirty = std::mem::take(&mut render.dirty);
    for cell in dirty {
        if let Some(e) = render.ents.remove(&cell) {
            commands.entity(e).despawn();
        }
        let Some(v) = ctx.terrain.cells.get(&cell).copied() else {
            continue;
        };
        let h = elev_height(grid.0.cell, v.elev);
        let c = grid::cell_center(&grid.0, cell);
        let (mesh, sxz) = match grid.0.kind {
            GridKind::Square => (ctx.lp.cube.clone(), grid.0.cell * 0.99),
            GridKind::HexFlat => (ctx.lp.hex_prism.clone(), grid.0.cell * 0.5 * 0.995),
        };
        let mat = ctx.mats.terrain(&mut ctx.materials, &assets, v.tex, v.elev);
        let e = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(c.x, h * 0.5 + 0.02, c.y).with_scale(Vec3::new(sxz, h, sxz)),
            ))
            .id();
        render.ents.insert(cell, e);
    }
}
