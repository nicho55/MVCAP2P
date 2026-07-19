use bevy::prelude::*;

use super::camera::CamRig;
use super::lowpoly::BASE_CELL;
use super::map::MapState;
use super::terrain::TerrainRender;
use super::tokens::Token;
use crate::protocol::*;

pub const SQRT3: f32 = 1.732_050_8;
const GRID_Y: f32 = 0.45;

#[derive(Resource, Default)]
pub struct GridRes(pub GridCfg);

/// Centro da célula em coordenadas de chão (x, z).
pub fn cell_center(g: &GridCfg, cell: Cell) -> Vec2 {
    match g.kind {
        GridKind::Square => Vec2::new(
            (cell.0 as f32 + 0.5) * g.cell,
            (cell.1 as f32 + 0.5) * g.cell,
        ),
        GridKind::HexFlat => {
            let s = g.cell * 0.5;
            Vec2::new(
                s * 1.5 * cell.0 as f32,
                s * SQRT3 * (cell.1 as f32 + cell.0 as f32 / 2.0),
            )
        }
    }
}

pub fn world_to_cell(g: &GridCfg, w: Vec2) -> Cell {
    match g.kind {
        GridKind::Square => ((w.x / g.cell).floor() as i32, (w.y / g.cell).floor() as i32),
        GridKind::HexFlat => {
            let s = g.cell * 0.5;
            let q = (2.0 / 3.0 * w.x) / s;
            let r = (-1.0 / 3.0 * w.x + SQRT3 / 3.0 * w.y) / s;
            axial_round(q, r)
        }
    }
}

fn axial_round(q: f32, r: f32) -> Cell {
    let (x, z) = (q, r);
    let y = -x - z;
    let (mut rx, ry, mut rz) = (x.round(), y.round(), z.round());
    let (dx, dy, dz) = ((rx - x).abs(), (ry - y).abs(), (rz - z).abs());
    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy <= dz {
        rz = -rx - ry;
    }
    (rx as i32, rz as i32)
}

pub fn hex_corners(g: &GridCfg, cell: Cell) -> [Vec2; 6] {
    let c = cell_center(g, cell);
    let s = g.cell * 0.5;
    core::array::from_fn(|i| {
        let a = (60.0 * i as f32).to_radians();
        c + Vec2::new(a.cos(), a.sin()) * s
    })
}

fn v3(p: Vec2) -> Vec3 {
    Vec3::new(p.x, GRID_Y, p.y)
}

/// Desenha o grid no chão (XZ) em volta do foco da câmera, limitado ao mapa.
pub fn draw_grid(
    mut gizmos: Gizmos,
    grid: Res<GridRes>,
    rig: Res<CamRig>,
    map_state: Res<MapState>,
) {
    let reach = (rig.dist * 1.3).clamp(400.0, 2600.0);
    let f = Vec2::new(rig.focus.x, rig.focus.z);
    let mut min = f - Vec2::splat(reach);
    let mut max = f + Vec2::splat(reach);
    if map_state.size.x > 0.0 {
        min = min.max(-map_state.size * 0.5);
        max = max.min(map_state.size * 0.5);
        if min.x >= max.x || min.y >= max.y {
            return;
        }
    }
    let col = Color::srgba(0.05, 0.04, 0.08, 0.5);
    match grid.0.kind {
        GridKind::Square => {
            let c = grid.0.cell;
            let x0 = (min.x / c).floor() as i32;
            let x1 = (max.x / c).ceil() as i32;
            let y0 = (min.y / c).floor() as i32;
            let y1 = (max.y / c).ceil() as i32;
            if (x1 - x0) + (y1 - y0) > 800 {
                return;
            }
            for i in x0..=x1 {
                gizmos.line(
                    v3(Vec2::new(i as f32 * c, min.y)),
                    v3(Vec2::new(i as f32 * c, max.y)),
                    col,
                );
            }
            for j in y0..=y1 {
                gizmos.line(
                    v3(Vec2::new(min.x, j as f32 * c)),
                    v3(Vec2::new(max.x, j as f32 * c)),
                    col,
                );
            }
        }
        GridKind::HexFlat => {
            let s = grid.0.cell * 0.5;
            let q0 = (min.x / (1.5 * s)).floor() as i32 - 1;
            let q1 = (max.x / (1.5 * s)).ceil() as i32 + 1;
            let rows = ((max.y - min.y) / (SQRT3 * s)).ceil() as i32 + 2;
            if (q1 - q0 + 1) * (rows + 1) > 4000 {
                return;
            }
            for q in q0..=q1 {
                let r0 = (min.y / (SQRT3 * s) - q as f32 / 2.0).floor() as i32 - 1;
                for dr in 0..=rows {
                    let cs = hex_corners(&grid.0, (q, r0 + dr));
                    gizmos.linestrip(
                        cs.iter().copied().map(v3).chain(std::iter::once(v3(cs[0]))),
                        col,
                    );
                }
            }
        }
    }
}

/// Quando o grid muda (tipo/tamanho de célula), reposiciona e reescala tokens
/// e redesenha o terreno. Filhos dos tokens escalam junto com o pai.
pub fn grid_reflow(
    grid: Res<GridRes>,
    mut trender: ResMut<TerrainRender>,
    mut q_tokens: Query<(&mut Transform, &Token)>,
) {
    if !grid.is_changed() {
        return;
    }
    let s = grid.0.cell / BASE_CELL;
    for (mut tf, tok) in &mut q_tokens {
        let c = cell_center(&grid.0, tok.meta.cell);
        tf.translation.x = c.x;
        tf.translation.z = c.y;
        tf.scale = Vec3::splat(s);
    }
    trender.full = true;
}
